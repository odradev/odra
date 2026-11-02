use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use odra::contract_def::HasIdent;
use odra::host::HostEnv;
use odra::schema::casper_contract_schema::Type;
use odra::schema::{
    readable_fields, ReadableField, SchemaCustomTypes, SchemaEvents, SchemaStorageLayout,
    StorageField, StorageKind, StorageLocation, CONTRACT_SCHEMA_VERSION
};
use odra::OdraContract;
use odra_schema::codec::{resolve_storage_from_text, CustomTypeSet};
use serde_derive::Serialize;

use crate::cmd::{CmdOutput, STORAGE_SUBCOMMAND};
use crate::container::ContractProvider;
use crate::{log, DeployedContractsContainer};

use super::OdraCommand;

const PATH_ARG: &str = "path";
const KEY_ARG: &str = "key";
const RAW_ARG: &str = "raw";

/// Reads the storage of a contract directly, without calling any entry point.
///
/// `storage <Contract>` prints the storage layout of the contract, `storage <Contract> <path>`
/// resolves the dotted field path against the layout and computes the storage key, so this works
/// before the contract is deployed too; the value is read from the host, and printed, only if the
/// contract is deployed.
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
            return Ok(StorageReport::Schema(StorageSchemaReport {
                casper_contract_schema_version: CONTRACT_SCHEMA_VERSION,
                key_name: contract.key_name.clone(),
                contract_name: contract.ident.clone(),
                storage: contract.layout.clone(),
                types: contract.custom_types.clone()
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
            .about("Reads the storage of a contract without calling it")
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
        custom_types.extend(
            <T as SchemaStorageLayout>::custom_types()
                .into_iter()
                .flatten()
        );

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
        let query = resolve_storage_from_text(&self.layout, path, keys)
            .map_err(|e| anyhow::anyhow!("Cannot resolve `{path}`: {e}"))?;

        let address = container.address_by_name(&self.key_name);

        let (value, raw_value) = match &address {
            None => (None, None),
            Some(address) => {
                let bytes = match &query.location {
                    StorageLocation::State { key } => {
                        env.get_storage_value(address, key.as_bytes())
                    }
                    StorageLocation::NamedKey { name } => env.get_named_value(address, name),
                    StorageLocation::Dictionary { name, key } => {
                        env.get_dictionary_value(address, name, key.as_bytes())
                    }
                };

                match bytes {
                    None => (None, None),
                    Some(bytes) if raw => (None, Some(hex::encode(&bytes))),
                    Some(bytes) => {
                        let (decoded, _) =
                            odra_schema::codec::decode(&bytes, &query.ty, &self.custom_types)
                                .map_err(|e| {
                                    anyhow::anyhow!("Cannot decode the stored value: {e}")
                                })?;
                        (Some(decoded), None)
                    }
                }
            }
        };

        Ok(StorageReport::Value(ValueReport {
            contract: self.key_name.clone(),
            path: path.to_string(),
            keys: keys.to_vec(),
            ty: odra_schema::codec::format_type_hint(&query.ty.0),
            location: query.location,
            deployed: address.is_some(),
            value,
            raw: raw_value
        }))
    }
}

/// The output of the `storage` command: either the contract's storage schema or a single value.
#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum StorageReport {
    Schema(StorageSchemaReport),
    Value(ValueReport)
}

/// The slice of the contract schema needed to describe a contract's storage: the same
/// `casper_contract_schema_version`, `storage` and `types` fields as a `*_schema.json` `odra-build`
/// writes, so this is a valid input to `odra-storage-reader`'s `StorageReader::from_json` too.
#[derive(Serialize)]
pub(crate) struct StorageSchemaReport {
    casper_contract_schema_version: u8,
    /// The name the contract is registered under.
    key_name: String,
    contract_name: String,
    storage: StorageKind,
    types: CustomTypeSet
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
    /// Whether the contract is deployed; `value` and `raw` are always `None` when it is not.
    #[serde(skip_serializing_if = "is_true")]
    deployed: bool,
    /// The decoded value, `None` if nothing is stored, the contract is not deployed, or
    /// `--raw` was used.
    value: Option<String>,
    /// The hex-encoded stored bytes, only with `--raw`.
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<String>
}

fn is_true(deployed: &bool) -> bool {
    *deployed
}

impl CmdOutput for StorageReport {
    fn pretty_print(&self) {
        match self {
            StorageReport::Schema(report) => {
                log(format!(
                    "Storage layout of {} ({}):",
                    report.key_name, report.contract_name
                ));
                print_fields(report.storage.fields(), 1);
                log(format!(
                    "Read with `storage {} <path> [-k <key>]...`:",
                    report.key_name
                ));
                for field in readable_fields(&report.storage) {
                    log(format!("  {}", format_readable_field(&field)));
                }
            }
            StorageReport::Value(report) => {
                log(format!("Contract: {}", report.contract));
                log(format!("Path:     {}", report.path));
                if !report.keys.is_empty() {
                    log(format!("Keys:     {}", report.keys.join(", ")));
                }
                log(format!("Type:     {}", report.ty));
                log(format!("Location: {}", format_location(&report.location)));
                match (report.deployed, &report.value, &report.raw) {
                    (false, _, _) => log("Value:    <contract not deployed>"),
                    (true, Some(value), _) => log(format!("Value:    {value}")),
                    (true, None, Some(raw)) => log(format!("Raw:      0x{raw}")),
                    (true, None, None) => log("Value:    <not set>")
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

/// A readable field as the arguments to read it, e.g. `balances -k <hash-...|account-hash-...>: DECIMAL`.
fn format_readable_field(field: &ReadableField) -> String {
    let hint = |t: &Type| odra_schema::codec::format_type_hint(&t.0);
    let keys: String = field
        .keys
        .iter()
        .map(|key| format!(" -k <{}>", hint(key)))
        .collect();
    format!("{}{keys}: {}", field.path, hint(&field.ty))
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
    let ty = |t: &Type| odra_schema::codec::format_type_hint(&t.0);
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
    fn formats_readable_fields() {
        use odra::schema::casper_contract_schema::NamedCLType;

        let layout = StorageKind::Module {
            fields: vec![
                StorageField::new("decimals", 1, StorageKind::value(NamedCLType::U8)),
                StorageField::new(
                    "allowances",
                    2,
                    StorageKind::Mapping {
                        key: Type(NamedCLType::Tuple2([
                            Box::new(NamedCLType::Key),
                            Box::new(NamedCLType::Key)
                        ])),
                        value: Box::new(StorageKind::Value {
                            ty: Type(NamedCLType::U256)
                        })
                    }
                ),
                StorageField::new(
                    "holders",
                    3,
                    StorageKind::List {
                        item: Type(NamedCLType::Key)
                    }
                ),
            ]
        };
        let lines: Vec<_> = readable_fields(&layout)
            .iter()
            .map(format_readable_field)
            .collect();
        assert_eq!(
            lines,
            vec![
                "decimals: 0-255|0x00|0b00000000",
                "allowances -k <hash-...|account-hash-...:hash-...|account-hash-...>: DECIMAL",
                "holders -k <UINT>: hash-...|account-hash-...",
                "holders.len: UINT",
            ]
        );
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
        assert!(matches!(report, StorageReport::Schema(_)));

        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(
            json["casper_contract_schema_version"],
            CONTRACT_SCHEMA_VERSION
        );
        assert_eq!(json["contract_name"], "TestContract");
        assert_eq!(json["storage"]["kind"], "module");
        assert!(json["types"].is_array());
    }

    #[test]
    fn resolves_a_path_without_a_deployed_contract() {
        use odra::schema::casper_contract_schema::NamedCLType;

        let contract = StorageContract {
            key_name: "Vault".to_string(),
            ident: "Vault".to_string(),
            layout: StorageKind::Module {
                fields: vec![StorageField::new(
                    "balance",
                    1,
                    StorageKind::value(NamedCLType::U256)
                )]
            },
            custom_types: CustomTypeSet::new()
        };

        let env = crate::test_utils::mock_host_env();
        let container = crate::test_utils::mock_contracts_container();
        let report = contract
            .read(&env, &container, "balance", &[], false)
            .unwrap();
        let StorageReport::Value(report) = report else {
            panic!("expected a value report")
        };

        assert!(!report.deployed);
        assert_eq!(report.value, None);
        assert_eq!(report.raw, None);
        assert_eq!(report.ty, "DECIMAL");
        assert!(matches!(report.location, StorageLocation::State { .. }));

        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["deployed"], false);
        assert!(json["value"].is_null());
    }
}
