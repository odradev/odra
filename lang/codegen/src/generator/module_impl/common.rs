use odra_ir::module::{Entrypoint, ImplItem};

pub fn to_entrypoints<'a>(methods: &'a [&'a ImplItem]) -> impl Iterator<Item = &'a dyn Entrypoint> {
    methods
        .iter()
        .filter_map(|item| match item {
            ImplItem::Method(method) => Some(vec![method as &dyn Entrypoint]),
            ImplItem::DelegationStatement(stmt) => {
                let entrypoints: Vec<&dyn Entrypoint> = stmt
                    .delegation_block
                    .functions
                    .iter()
                    .map(|f| f as &dyn Entrypoint)
                    .collect();
                Some(entrypoints)
            }
            _ => None
        })
        .flatten()
}
