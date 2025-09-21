use actix_web::web::Data;
use juniper::{EmptyMutation, EmptySubscription, GraphQLType, GraphQLTypeAsync, RootNode};
use r2d2::Pool;
use redis::Client;

use crate::repos::graphql::{fine::FineRepo, loan::LoanRepo, payment::PaymentRepo};
use crate::repos::graphql::cuota::CuotaRepo;

//Context Related
#[derive(Clone)]
pub struct GeneralContext {
    pub pool: Data<Pool<Client>>,
}

impl GeneralContext {
    pub fn payment_repo(&self) -> PaymentRepo {
        PaymentRepo {
            pool: self.pool.clone(),
        }
    }
    pub fn loan_repo(&self) -> LoanRepo {
        LoanRepo {
            pool: self.pool.clone(),
        }
    }
    pub fn fine_repo(&self) -> FineRepo {
        FineRepo {
            pool: self.pool.clone(),
        }
    }
    pub fn cuota_repo(&self) -> CuotaRepo {
        CuotaRepo {
            pool: self.pool.clone(),
        }
    }
}

//I don't like this rust boilerplate, but meh, Ig rust doesn't adapt that good to abstractions
impl juniper::Context for GeneralContext {}

//Schema Related
pub type GeneralSchema<T> =
    RootNode<'static, T, EmptyMutation<GeneralContext>, EmptySubscription<GeneralContext>>;

pub fn create_schema<GenericQuery>(query: GenericQuery) -> Data<GeneralSchema<GenericQuery>>
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
    Data::new(schema)
}
