use juniper::{EmptyMutation, EmptySubscription, RootNode};

pub type GeneralSchema<T> = RootNode<'static, T, EmptyMutation<T>, EmptySubscription<T>>;
