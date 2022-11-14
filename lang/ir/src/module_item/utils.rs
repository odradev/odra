pub fn is_mut(sig: &syn::Signature) -> bool {
    let receiver = sig.inputs.iter().find_map(|input| match input {
        syn::FnArg::Receiver(receiver) => Some(receiver),
        syn::FnArg::Typed(_) => None
    });
    receiver.and_then(|r| r.mutability).is_some()
}
