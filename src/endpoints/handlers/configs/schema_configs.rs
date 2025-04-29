use actix_web::web;
use juniper::{EmptyMutation, EmptySubscription, GraphQLType, GraphQLTypeAsync, RootNode};

use crate::repos::graphql::{loan::LoanRepo, payment::PaymentRepo};

//Context Related
#[derive(Clone)]
pub struct GeneralContext {}

impl GeneralContext {
    pub fn payment_repo(&self) -> PaymentRepo {
        return PaymentRepo::init();
    }
    pub fn loan_repo(&self) -> LoanRepo {
        return LoanRepo::init();
    }
}

//I don't like this rust boilerplate, but meh, Ig rust doesn't adapt that good to abstractions
impl juniper::Context for GeneralContext {}

//Schema Related
pub type GeneralSchema<T> =
    RootNode<'static, T, EmptyMutation<GeneralContext>, EmptySubscription<GeneralContext>>;

pub fn create_schema<GenericQuery>(query: GenericQuery) -> web::Data<GeneralSchema<GenericQuery>>
where
    //Here we are putting specifics Types
    GenericQuery: GraphQLTypeAsync<Context = GeneralContext, TypeInfo = ()>
        //Also here in the context, a Trait with that specific Type/Struct
        + GraphQLType<Context = GeneralContext>
        + Send
        + Sync,
    GenericQuery::TypeInfo: Send + Sync,
{
    let schema = RootNode::new(query, EmptyMutation::new(), EmptySubscription::new());

    // I always need for passing the squema to actix
    return web::Data::new(schema);
}
