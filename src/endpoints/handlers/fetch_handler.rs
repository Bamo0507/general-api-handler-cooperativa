use actix_web::web;
use juniper::{EmptyMutation, EmptySubscription, RootNode};

use crate::repos::graphql::demo_repo::Demo;

//* All context and repos

#[derive(Clone)]
pub struct Context {}

impl Context {
    fn demo_repo(&self) -> Demo {
        return Demo::init();
    }
}

//* Queries

//I don't like this rust boilerplate, but meh, Ig rust doesn't adapt that good to abstractions
impl juniper::Context for Context {}

pub struct Query {}

#[juniper::graphql_object(
    Context = Context,
)]
impl Query {
    //TODO: add the necesary possible queries
    pub async fn demo(context: &Context) -> Demo {
        return context.demo_repo();
    }
}

//* Schemas side
pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn create_schema() -> web::Data<Schema> {
    let schema = RootNode::new(Query {}, EmptyMutation::new(), EmptySubscription::new());

    // I always need for passing the squema to actix
    return web::Data::new(schema);
}
