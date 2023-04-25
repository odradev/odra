use casper_types::{CLType, CLTyped};
use odra_types::Type;

/// A type which can be described as a [Type].
pub trait Typed {
    fn ty() -> Type;
}

impl<T: CLTyped> Typed for T {
    fn ty() -> Type {
        cl_type_to_type(T::cl_type())
    }
}

fn cl_type_to_type(ty: CLType) -> Type {
    match ty {
        CLType::Bool => Type::Bool,
        CLType::I32 => Type::I32,
        CLType::I64 => Type::I64,
        CLType::U8 => Type::U8,
        CLType::U32 => Type::U32,
        CLType::U64 => Type::U64,
        CLType::U128 => Type::U128,
        CLType::U256 => Type::U256,
        CLType::U512 => Type::U512,
        CLType::Unit => Type::Unit,
        CLType::String => Type::String,
        CLType::Option(ty) => Type::Option(boxed_cl_type_to_boxed_type(ty)),
        CLType::List(ty) => Type::Vec(boxed_cl_type_to_boxed_type(ty)),
        CLType::Result { ok, err } => Type::Map {
            key: boxed_cl_type_to_boxed_type(ok),
            value: boxed_cl_type_to_boxed_type(err)
        },
        CLType::Map { key, value } => Type::Map {
            key: boxed_cl_type_to_boxed_type(key),
            value: boxed_cl_type_to_boxed_type(value)
        },
        CLType::Tuple1(types) => Type::Tuple1(types.map(boxed_cl_type_to_boxed_type)),
        CLType::Tuple2(types) => Type::Tuple2(types.map(boxed_cl_type_to_boxed_type)),
        CLType::Tuple3(types) => Type::Tuple3(types.map(boxed_cl_type_to_boxed_type)),
        CLType::Any => Type::Any,
        CLType::Key => Type::Address,
        CLType::ByteArray(b) => Type::ByteArray(b),
        _ => panic!("Unsupported type {:?}", ty)
    }
}

#[allow(clippy::boxed_local)]
fn boxed_cl_type_to_boxed_type(ty: Box<CLType>) -> Box<Type> {
    Box::new(cl_type_to_type(*ty))
}
