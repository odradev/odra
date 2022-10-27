mod address;
mod call_args;
mod mock_vm_type;
mod uints;

pub use address::Address;
pub use call_args::CallArgs;
pub use mock_vm_type::{MockVMSerializationError, MockVMType};
pub use uints::{U256, U512};

pub type Balance = U256;
pub type BlockTime = u64;

pub trait OdraType: MockVMType {}
impl<T: MockVMType> OdraType for T {}
