use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use odra::contract_def::HasIdent;
use odra::host::HostEnv;
use odra::schema::casper_contract_schema::Type;
use odra::schema::{
    resolve_storage_with, SchemaCustomTypes, SchemaEvents, SchemaStorageLayout, StorageField,
    StorageKind, StorageLocation
};
use odra::OdraContract;
use serde_derive::Serialize;

use crate::cmd::{CmdOutput, STORAGE_SUBCOMMAND};
use crate::container::ContractProvider;
use crate::custom_types::CustomTypeSet;
use crate::{log, types, DeployedContractsContainer};

use super::OdraCommand;

const PATH_ARG: &str = "path";
const KEY_ARG: &str = "key";
const RAW_ARG: &str = "raw";

/// Reads the storage of a deployed contract directly, without calling any entry point.
///
/// `storage <Contract>` prints the storage layout of the contract, `storage <Contract> <path>`
/// resolves the dotted field path against the layout, computes the storage key and reads the
/// value from the host.
#[derive(Default)]
pub(crate) struct StorageCmd {
    contracts: Vec<StorageContract>
}

impl StorageCmd {
    pub fn add_contract<
        T: SchemaStorageLayout + SchemaCustomTypes + SchemaEvents + OdraContract
    >(
        &mut self
    ) {
        self.contracts.push(StorageContract::new::<T>(None));
    }

    pub fn add_contract_named<
        T: SchemaStorageLayout + SchemaCustomTypes + SchemaEvents + OdraContract
    >(
        &mut self,
        key_name: String
    ) {
        self.contracts
            .push(StorageContract::new::<T>(Some(key_name)));
    }

    fn available(&self) -> String {
        self.contracts
            .iter()
            .map(|c| c.key_name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl OdraCommand for StorageCmd {
    type Output = StorageReport;

    fn exec(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        _types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        let (name, args) = args
            .subcommand()
            .ok_or_else(|| anyhow::anyhow!("No contract given. Available: {}", self.available()))?;
        let contract = self
            .contracts
            .iter()
            .find(|c| c.key_name == name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No contract '{name}' registered. Available: {}",
                    self.available()
                )
            })?;

        let Some(path) = args.get_one::<String>(PATH_ARG) else {
            return Ok(StorageReport::Layout(LayoutReport {
                contract: contract.key_name.clone(),
                ident: contract.ident.clone(),
                layout: contract.layout.clone()
            }));
        };

        let keys = args
            .get_many::<String>(KEY_ARG)
            .map(|k| k.cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let raw = args.get_flag(RAW_ARG);
        contract.read(env, container, path, &keys, raw)
    }
}

impl From<&StorageCmd> for Command {
    fn from(value: &StorageCmd) -> Self {
        let subcommands = value.contracts.iter().map(|c| {
            Command::new(&c.key_name)
                .about(format!("Reads the storage of the {} contract", c.key_name))
                .arg(
                    Arg::new(PATH_ARG)
                        .help("Dotted path to a field, e.g. `erc20.balances`. Omit it to print the storage layout.")
                        .required(false)
                )
                .arg(
                    Arg::new(KEY_ARG)
                        .long(KEY_ARG)
                        .short('k')
                        .help("Key of a Mapping, List item or dictionary on the path. Repeat it for nested containers, in path order.")
                        .action(ArgAction::Append)
                )
                .arg(
                    Arg::new(RAW_ARG)
                        .long(RAW_ARG)
                        .help("Print the stored bytes hex-encoded instead of decoding them.")
                        .action(ArgAction::SetTrue)
                )
        });
        Command::new(STORAGE_SUBCOMMAND)
            .about("Reads the storage of a deployed contract without calling it")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommands(subcommands)
    }
}

struct StorageContract {
    key_name: String,
    ident: String,
    layout: StorageKind,
    custom_types: CustomTypeSet
}

impl StorageContract {
    fn new<T: SchemaStorageLayout + SchemaCustomTypes + SchemaEvents + OdraContract>(
        key_name: Option<String>
    ) -> Self {
        let ident = T::HostRef::ident();
        let key_name = key_name.unwrap_or_else(|| ident.clone());

        let mut custom_types = CustomTypeSet::new();
        custom_types.extend(T::schema_types().into_iter().flatten());
        custom_types.extend(<T as SchemaEvents>::custom_types().into_iter().flatten());

        Self {
            key_name,
            ident,
            layout: T::storage_kind(),
            custom_types
        }
    }

    fn read(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        path: &str,
        keys: &[String],
        raw: bool
    ) -> Result<StorageReport> {
        let address = container
            .address_by_name(&self.key_name)
            .ok_or_else(|| anyhow::anyhow!("Contract '{}' is not deployed", self.key_name))?;

        let mut pending_keys = keys.iter();
        let query = resolve_storage_with(&self.layout, path, |ty: &Type| {
            let Some(key) = pending_keys.next() else {
                return Ok(None);
            };
            types::into_bytes(&ty.0, key).map(Some).map_err(|e| {
                format!(
                    "cannot parse `{key}` as {}: {e}",
                    types::format_type_hint(&ty.0)
                )
            })
        })
        .map_err(|e| anyhow::anyhow!("Cannot resolve `{path}`: {e}"))?;
        let unused = pending_keys.count();
        if unused > 0 {
            anyhow::bail!("Cannot resolve `{path}`: {unused} key(s) were given but not used");
        }

        let bytes = match &query.location {
            StorageLocation::State { key } => env.get_storage_value(&address, key.as_bytes()),
            StorageLocation::NamedKey { name } => env.get_named_value(&address, name),
            StorageLocation::Dictionary { name, key } => {
                env.get_dictionary_value(&address, name, key.as_bytes())
            }
        };

        let (value, raw_value) = match bytes {
            None => (None, None),
            Some(bytes) if raw => (None, Some(hex::encode(&bytes))),
            Some(bytes) => {
                let (decoded, _) = types::decode(&bytes, &query.ty, &self.custom_types)
                    .map_err(|e| anyhow::anyhow!("Cannot decode the stored value: {e}"))?;
                (Some(decoded), None)
            }
        };

        Ok(StorageReport::Value(ValueReport {
            contract: self.key_name.clone(),
            path: path.to_string(),
            keys: keys.to_vec(),
            ty: types::format_type_hint(&query.ty.0),
            location: query.location,
            value,
            raw: raw_value
        }))
    }
}

/// The output of the `storage` command: either the layout or a single value.
#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum StorageReport {
    Layout(LayoutReport),
    Value(ValueReport)
}

#[derive(Serialize)]
pub(crate) struct LayoutReport {
    contract: String,
    ident: String,
    layout: StorageKind
}

#[derive(Serialize)]
pub(crate) struct ValueReport {
    contract: String,
    path: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    keys: Vec<String>,
    #[serde(rename = "type")]
    ty: String,
    #[serde(flatten)]
    location: StorageLocation,
    /// The decoded value, `None` if nothing is stored (or `--raw` was used).
    value: Option<String>,
    /// The hex-encoded stored bytes, only with `--raw`.
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<String>
}

impl CmdOutput for StorageReport {
    fn pretty_print(&self) {
        match self {
            StorageReport::Layout(report) => {
                log(format!(
                    "Storage layout of {} ({}):",
                    report.contract, report.ident
                ));
                print_fields(report.layout.fields(), 1);
            }
            StorageReport::Value(report) => {
                log(format!("Contract: {}", report.contract));
                log(format!("Path:     {}", report.path));
                if !report.keys.is_empty() {
                    log(format!("Keys:     {}", report.keys.join(", ")));
                }
                log(format!("Type:     {}", report.ty));
                log(format!("Location: {}", format_location(&report.location)));
                match (&report.value, &report.raw) {
                    (Some(value), _) => log(format!("Value:    {value}")),
                    (None, Some(raw)) => log(format!("Raw:      0x{raw}")),
                    (None, None) => log("Value:    <not set>")
                }
            }
        }
    }
}

fn format_location(location: &StorageLocation) -> String {
    match location {
        StorageLocation::State { key } => format!("state[{key}]"),
        StorageLocation::NamedKey { name } => format!("named key `{name}`"),
        StorageLocation::Dictionary { name, key } => format!("dictionary `{name}`[{key}]")
    }
}

/// Prints the fields of a module as an indented tree.
pub(crate) fn print_fields(fields: &[StorageField], depth: usize) {
    let indent = "  ".repeat(depth);
    for field in fields {
        let (desc, nested) = describe(&field.kind);
        log(format!("{indent}{} [{}]{desc}", field.name, field.index));
        if let Some(nested) = nested {
            print_fields(nested, depth + 1);
        }
    }
}

/// Describes a storage kind in one line and returns the nested fields to print below, if any.
fn describe(kind: &StorageKind) -> (String, Option<&[StorageField]>) {
    let ty = |t: &Type| types::format_type_hint(&t.0);
    match kind {
        StorageKind::Value { ty: t } => (format!(": {}", ty(t)), None),
        StorageKind::List { item } => (format!(": List<{}>", ty(item)), None),
        StorageKind::Sequence { ty: t } => (format!(": Sequence<{}>", ty(t)), None),
        StorageKind::Module { fields } => (String::new(), Some(fields)),
        StorageKind::NamedKey { name, ty: t } => {
            (format!(": {} (named key `{name}`)", ty(t)), None)
        }
        StorageKind::Dictionary {
            name, key, value, ..
        } => (
            format!(
                ": Dictionary<{}, {}> (dictionary `{name}`)",
                ty(key),
                ty(value)
            ),
            None
        ),
        StorageKind::Mapping { key, value } => match value.as_ref() {
            StorageKind::Module { fields } => {
                (format!(": Mapping<{}, module>", ty(key)), Some(fields))
            }
            other => {
                let (inner, nested) = describe(other);
                let inner = inner.trim_start_matches(": ");
                (format!(": Mapping<{}, {inner}>", ty(key)), nested)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestContract;

    #[test]
    fn builds_storage_command_with_subcommands() {
        let mut cmd = StorageCmd::default();
        cmd.add_contract::<TestContract>();
        cmd.add_contract_named::<TestContract>("Second".to_string());

        let clap_cmd: Command = (&cmd).into();
        assert_eq!(clap_cmd.get_name(), STORAGE_SUBCOMMAND);
        let names: Vec<_> = clap_cmd.get_subcommands().map(|c| c.get_name()).collect();
        assert_eq!(names, vec!["TestContract", "Second"]);

        let sub = clap_cmd.find_subcommand("TestContract").unwrap();
        assert!(sub.get_arguments().any(|a| a.get_id() == PATH_ARG));
        assert!(sub.get_arguments().any(|a| a.get_id() == KEY_ARG));
        assert!(sub.get_arguments().any(|a| a.get_id() == RAW_ARG));
    }

    #[test]
    fn describes_storage_kinds() {
        use odra::schema::casper_contract_schema::NamedCLType;
        use odra::schema::KeyEncoding;

        let ty = |t: NamedCLType| Type(t);
        let module = StorageKind::Module {
            fields: vec![StorageField::new(
                "owner",
                1,
                StorageKind::Value {
                    ty: ty(NamedCLType::Option(Box::new(NamedCLType::Key)))
                }
            )]
        };

        let cases = vec![
            (
                StorageKind::Value {
                    ty: ty(NamedCLType::U256)
                },
                ": DECIMAL",
                false
            ),
            (
                StorageKind::Mapping {
                    key: ty(NamedCLType::Key),
                    value: Box::new(StorageKind::Value {
                        ty: ty(NamedCLType::U256)
                    })
                },
                ": Mapping<hash-...|account-hash-..., DECIMAL>",
                false
            ),
            (
                StorageKind::Mapping {
                    key: ty(NamedCLType::U8),
                    value: Box::new(module.clone())
                },
                ": Mapping<0-255|0x00|0b00000000, module>",
                true
            ),
            (
                StorageKind::List {
                    item: ty(NamedCLType::U32)
                },
                ": List<UINT>",
                false
            ),
            (
                StorageKind::Sequence {
                    ty: ty(NamedCLType::U64)
                },
                ": Sequence<UINT>",
                false
            ),
            (module.clone(), "", true),
            (
                StorageKind::NamedKey {
                    name: "decimals".to_string(),
                    ty: ty(NamedCLType::U8)
                },
                ": 0-255|0x00|0b00000000 (named key `decimals`)",
                false
            ),
            (
                StorageKind::Dictionary {
                    name: "balances".to_string(),
                    key: ty(NamedCLType::Key),
                    key_encoding: KeyEncoding::Base64,
                    value: ty(NamedCLType::U256)
                },
                ": Dictionary<hash-...|account-hash-..., DECIMAL> (dictionary `balances`)",
                false
            ),
        ];

        for (kind, expected, has_nested) in cases {
            let (desc, nested) = describe(&kind);
            assert_eq!(desc, expected);
            assert_eq!(nested.is_some(), has_nested);
        }
    }

    #[test]
    fn prints_layout_without_path() {
        let mut cmd = StorageCmd::default();
        cmd.add_contract::<TestContract>();
        let clap_cmd: Command = (&cmd).into();
        let matches = clap_cmd
            .try_get_matches_from(vec![STORAGE_SUBCOMMAND, "TestContract"])
            .unwrap();

        let env = crate::test_utils::mock_host_env();
        let container = crate::test_utils::mock_contracts_container();
        let report = cmd
            .exec(&env, &matches, &CustomTypeSet::new(), &container)
            .unwrap();
        assert!(matches!(report, StorageReport::Layout(_)));
    }
}
