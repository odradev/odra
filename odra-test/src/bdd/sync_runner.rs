#![allow(dead_code)]
use cucumber::{cli, codegen::WorldInventory, event, parser, step, Event, World};
use derive_more::{Display, From};
use futures::{
    executor::block_on,
    future,
    stream::{self, LocalBoxStream},
    FutureExt, Stream, StreamExt, TryStreamExt
};
use odra_core::prelude::*;
use std::{fmt::Debug, panic::AssertUnwindSafe, sync::Arc, thread};

type ScenarioHook = fn(gherkin::Scenario);

#[derive(Default)]
pub struct SyncRunner<W: World + WorldInventory + Debug + Send> {
    _phantom: core::marker::PhantomData<W>,
    before_scenario: Option<ScenarioHook>,
    after_scenario: Option<ScenarioHook>
}

impl<W: World + WorldInventory + Debug + Send + Default> SyncRunner<W> {
    /// Creates a new [`SyncRunner<W>`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a hook to be executed before each scenario.
    pub fn with_before_scenario(mut self, hook: ScenarioHook) -> Self {
        self.before_scenario = Some(hook);
        self
    }

    /// Sets a hook to be executed after each scenario.
    pub fn with_after_scenario(mut self, hook: ScenarioHook) -> Self {
        self.after_scenario = Some(hook);
        self
    }
}

impl<W> SyncRunner<W>
where
    W: World + WorldInventory + Debug + Clone + Send + Sync + 'static,
    <W as World>::Error: Debug
{
    fn execute_all<S>(
        features: S,
        before: Option<ScenarioHook>,
        after: Option<ScenarioHook>
    ) -> LocalBoxStream<'static, parser::Result<Event<event::Cucumber<W>>>>
    where
        S: Stream<Item = parser::Result<gherkin::Feature>> + 'static
    {
        stream::once(future::ok(event::Cucumber::Started))
            .chain(
                features
                    .map_ok(move |f| Self::execute_feature(f, before, after).map(Ok))
                    .try_flatten()
            )
            .chain(stream::once(future::ok(event::Cucumber::Finished)))
            .map_ok(Event::new)
            .boxed_local()
    }

    fn execute_feature(
        feature: gherkin::Feature,
        before: Option<ScenarioHook>,
        after: Option<ScenarioHook>
    ) -> impl Stream<Item = event::Cucumber<W>> {
        let feature = Arc::new(feature);
        let background = feature.background.clone();
        stream::once(future::ready(event::Feature::Started))
            .chain(
                stream::iter(feature.scenarios.clone())
                    .then(move |s| {
                        Self::execute_feature_scenario(s, background.clone(), before, after)
                    })
                    .flatten()
            )
            .chain(
                stream::iter(feature.rules.clone())
                    .then(move |r| Self::execute_rule(r, before, after))
                    .flatten()
            )
            .chain(stream::once(future::ready(event::Feature::Finished)))
            .map(move |ev| event::Cucumber::Feature(feature.clone(), ev))
    }

    async fn execute_rule(
        rule: gherkin::Rule,
        before: Option<ScenarioHook>,
        after: Option<ScenarioHook>
    ) -> impl Stream<Item = event::Feature<W>> {
        let rule = Arc::new(rule);
        let background = rule.background.clone();
        stream::once(future::ready(event::Rule::Started))
            .chain(
                stream::iter(rule.scenarios.clone())
                    .then(move |s| {
                        Self::execute_rule_scenario(s, background.clone(), before, after)
                    })
                    .flatten()
            )
            .chain(stream::once(future::ready(event::Rule::Finished)))
            .map(move |ev| event::Feature::Rule(rule.clone(), ev))
    }

    async fn execute_feature_scenario(
        scenario: gherkin::Scenario,
        background: Option<gherkin::Background>,
        before: Option<ScenarioHook>,
        after: Option<ScenarioHook>
    ) -> impl Stream<Item = event::Feature<W>> {
        let steps = Self::execute_scenario(scenario.clone(), background, before, after).await;
        let scenario = Arc::new(scenario);

        stream::once(future::ready(event::Scenario::Started))
            .chain(stream::iter(steps.into_iter().flat_map(|(step, ev)| {
                let step = Arc::new(step);
                [
                    event::Scenario::Step(step.clone(), event::Step::Started),
                    event::Scenario::Step(step, ev)
                ]
            })))
            .chain(stream::once(future::ready(event::Scenario::Finished)))
            .map(move |event| {
                event::Feature::Scenario(
                    scenario.clone(),
                    event::RetryableScenario {
                        event,
                        retries: None
                    }
                )
            })
    }

    async fn execute_rule_scenario(
        scenario: gherkin::Scenario,
        background: Option<gherkin::Background>,
        before: Option<ScenarioHook>,
        after: Option<ScenarioHook>
    ) -> impl Stream<Item = event::Rule<W>> {
        let steps = Self::execute_scenario(scenario.clone(), background, before, after).await;
        let scenario = Arc::new(scenario);

        stream::once(future::ready(event::Scenario::Started))
            .chain(stream::iter(steps.into_iter().flat_map(|(step, ev)| {
                let step = Arc::new(step);
                [
                    event::Scenario::Step(step.clone(), event::Step::Started),
                    event::Scenario::Step(step, ev)
                ]
            })))
            .chain(stream::once(future::ready(event::Scenario::Finished)))
            .map(move |event| {
                event::Rule::Scenario(
                    scenario.clone(),
                    event::RetryableScenario {
                        event,
                        retries: None
                    }
                )
            })
    }

    async fn execute_scenario(
        scenario: gherkin::Scenario,
        background: Option<gherkin::Background>,
        before: Option<ScenarioHook>,
        after: Option<ScenarioHook>
    ) -> Vec<(gherkin::Step, event::Step<W>)> {
        let hook_scenario = scenario.clone();
        let scenario = Arc::new(scenario);
        let s = scenario.clone();
        let (mut tx, mut rx) = futures::channel::mpsc::channel(1);

        thread::spawn(move || {
            let steps = block_on(async {
                if let Some(before) = before {
                    before(hook_scenario.clone());
                };
                let mut steps = Vec::new();
                let mut world = W::new().await.unwrap();
                let mut can_run_scenario = true;
                if let Some(background) = background {
                    for step in background.steps {
                        let (w, ev) = Self::execute_step(world, step.clone()).await;
                        world = w;
                        let should_stop = matches!(ev, SyncStep::Failed(..));
                        steps.push((step, ev));
                        if should_stop {
                            can_run_scenario = false;
                            break;
                        }
                    }
                }

                if can_run_scenario {
                    for step in s.steps.clone() {
                        let (w, ev) = Self::execute_step(world, step.clone()).await;
                        world = w;
                        let should_stop = matches!(ev, SyncStep::Failed(..));
                        steps.push((step, ev));
                        if should_stop {
                            break;
                        }
                    }
                }
                if let Some(after) = after {
                    after(hook_scenario);
                };
                steps
            });
            tx.try_send(steps).unwrap();
        });

        let steps = rx.next().await.unwrap();
        let steps: Vec<(gherkin::Step, event::Step<W>)> = steps
            .into_iter()
            .map(|(step, ev)| (step, event::Step::from(ev)))
            .collect();
        steps
    }

    async fn execute_step(mut world: W, step: gherkin::Step) -> (W, SyncStep<W>) {
        let ev = if let Some((step_fn, captures, loc, ctx)) =
            W::collection().find(&step).expect("Ambiguous match")
        {
            // Panic represents a failed assertion in a step matching
            // function.
            match AssertUnwindSafe(step_fn(&mut world, ctx))
                .catch_unwind()
                .await
            {
                Ok(()) => SyncStep::Passed(captures, loc),
                Err(e) => SyncStep::Failed(
                    Some(captures),
                    loc,
                    Some(Arc::new(world.clone())),
                    SyncStepError::Panic(e.downcast_ref::<String>().cloned().unwrap())
                )
            }
        } else {
            SyncStep::Skipped
        };
        (world, ev)
    }
}

impl<W> cucumber::Runner<W> for SyncRunner<W>
where
    W: World + WorldInventory + Debug + Clone + Send + Sync,
    <W as World>::Error: Debug
{
    type Cli = cli::Empty; // we provide no CLI options
    type EventStream = LocalBoxStream<'static, parser::Result<Event<event::Cucumber<W>>>>;

    fn run<S>(self, features: S, _: Self::Cli) -> Self::EventStream
    where
        S: Stream<Item = parser::Result<gherkin::Feature>> + 'static
    {
        Self::execute_all(features, self.before_scenario, self.after_scenario)
    }
}

#[derive(Debug)]
pub enum SyncStep<World> {
    Started,
    Skipped,
    Passed(regex::CaptureLocations, Option<step::Location>),
    Failed(
        Option<regex::CaptureLocations>,
        Option<step::Location>,
        Option<Arc<World>>,
        SyncStepError
    )
}

#[derive(Clone, Debug, Display, From)]
pub enum SyncStepError {
    #[display(fmt = "Step doesn't match any function")]
    NotFound,
    #[display(fmt = "Step match is ambiguous: {}", _0)]
    AmbiguousMatch(step::AmbiguousMatchError),
    #[display(fmt = "Step panicked. Captured output: {}", _0)]
    Panic(String)
}

impl<W> From<SyncStep<W>> for cucumber::event::Step<W> {
    fn from(value: SyncStep<W>) -> Self {
        match value {
            SyncStep::Started => cucumber::event::Step::Started,
            SyncStep::Skipped => cucumber::event::Step::Skipped,
            SyncStep::Passed(capture_location, maybe_location) => {
                cucumber::event::Step::Passed(capture_location, maybe_location)
            }
            SyncStep::Failed(capture_location, maybe_location, world, err) => {
                cucumber::event::Step::Failed(capture_location, maybe_location, world, err.into())
            }
        }
    }
}

impl From<SyncStepError> for cucumber::event::StepError {
    fn from(value: SyncStepError) -> Self {
        match value {
            SyncStepError::NotFound => cucumber::event::StepError::NotFound,
            SyncStepError::AmbiguousMatch(err) => cucumber::event::StepError::AmbiguousMatch(err),
            SyncStepError::Panic(msg) => cucumber::event::StepError::Panic(Arc::new(msg))
        }
    }
}
