use crate::{
    components::{
        delete_user::DeleteUser,
        router::{AppRoute, Link},
    },
    infra::common_component::{CommonComponent, CommonComponentParts},
};
use anyhow::{Error, Result};
use graphql_client::GraphQLQuery;
use yew::prelude::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../schema.graphql",
    query_path = "queries/list_users.graphql",
    response_derives = "Debug",
    custom_scalars_module = "crate::infra::graphql"
)]
pub struct ListUsersQuery;

use list_users_query::{RequestFilter, ResponseData};

type User = list_users_query::ListUsersQueryUsers;

pub struct UserTable {
    common: CommonComponentParts<Self>,
    users: Option<Vec<User>>,
    search_query: String,
}

pub enum Msg {
    ListUsersResponse(Result<ResponseData>),
    OnUserDeleted(String),
    OnError(Error),
    OnSearchChange(String),
    SearchUsers,
}

impl CommonComponent<UserTable> for UserTable {
    fn handle_msg(&mut self, ctx: &Context<Self>, msg: <Self as Component>::Message) -> Result<bool> {
        match msg {
            Msg::ListUsersResponse(users) => {
                self.users = Some(users?.users.into_iter().collect());
                Ok(true)
            }
            Msg::OnError(e) => Err(e),
            Msg::OnUserDeleted(user_id) => {
                debug_assert!(self.users.is_some());
                self.users.as_mut().unwrap().retain(|u| u.id != user_id);
                Ok(true)
            }
            Msg::OnSearchChange(query) => {
                self.search_query = query;
                Ok(false)
            }
            Msg::SearchUsers => {
                let filter = if self.search_query.is_empty() {
                    None
                } else {
                    Some(RequestFilter {
                        any: Box::new(Some(vec![
                            RequestFilter {
                                eq: Some(list_users_query::EqualityConstraint {
                                    field: "id".to_string(),
                                    value: self.search_query.clone(),
                                }),
                                all: Box::new(None),
                                any: Box::new(None),
                                not: Box::new(None),
                                memberOf: None,
                                memberOfId: None,
                            },
                            RequestFilter {
                                eq: Some(list_users_query::EqualityConstraint {
                                    field: "email".to_string(),
                                    value: self.search_query.clone(),
                                }),
                                all: Box::new(None),
                                any: Box::new(None),
                                not: Box::new(None),
                                memberOf: None,
                                memberOfId: None,
                            },
                            RequestFilter {
                                eq: Some(list_users_query::EqualityConstraint {
                                    field: "displayName".to_string(),
                                    value: self.search_query.clone(),
                                }),
                                all: Box::new(None),
                                any: Box::new(None),
                                not: Box::new(None),
                                memberOf: None,
                                memberOfId: None,
                            }
                        ])),
                        all: Box::new(None),
                        not: Box::new(None),
                        eq: None,
                        memberOf: None,
                        memberOfId: None,
                    })
                };
                self.get_users(ctx, filter);
                Ok(true)
            }
        }
    }

    fn mut_common(&mut self) -> &mut CommonComponentParts<Self> {
        &mut self.common
    }
}

impl UserTable {
    fn get_users(&mut self, ctx: &Context<Self>, req: Option<RequestFilter>) {
        self.common.call_graphql::<ListUsersQuery, _>(
            ctx,
            list_users_query::Variables { filters: req },
            Msg::ListUsersResponse,
            "Error trying to fetch users",
        );
    }
}

impl Component for UserTable {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut table = UserTable {
            common: CommonComponentParts::<Self>::create(),
            users: None,
            search_query: String::new(),
        };
        table.get_users(ctx, None);
        table
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        CommonComponentParts::<Self>::update(self, ctx, msg)
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
              {self.view_search_bar(ctx)}
              {self.view_users(ctx)}
              {self.view_errors()}
            </div>
        }
    }
}

impl UserTable {
    fn view_search_bar(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
            <div class="card mb-3">
                <div class="card-body">
                    <h5 class="card-title">{"Search Users"}</h5>
                    <div class="row g-3 align-items-center">
                        <div class="col-auto">
                            <input 
                                type="text" 
                                class="form-control" 
                                placeholder="Search by ID, email, or display name"
                                value={self.search_query.clone()}
                                oninput={link.callback(|e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    Msg::OnSearchChange(input.value())
                                })}
                                onkeypress={link.callback(|e: KeyboardEvent| {
                                    if e.key() == "Enter" {
                                        Msg::SearchUsers
                                    } else {
                                        Msg::OnSearchChange("".to_string())
                                    }
                                })}
                            />
                        </div>
                        <div class="col-auto">
                            <button 
                                class="btn btn-primary"
                                onclick={link.callback(|_| Msg::SearchUsers)}
                                disabled={self.common.is_task_running()}
                            >
                                <i class="bi-search me-2"></i>
                                {"Search"}
                            </button>
                        </div>
                        {
                            if !self.search_query.is_empty() {
                                html! {
                                    <div class="col-auto">
                                        <button 
                                            class="btn btn-secondary"
                                            onclick={link.batch_callback(|_| vec![
                                                Msg::OnSearchChange("".to_string()),
                                                Msg::SearchUsers
                                            ])}
                                        >
                                            <i class="bi-x-circle me-2"></i>
                                            {"Clear"}
                                        </button>
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>
            </div>
        }
    }

    fn view_users(&self, ctx: &Context<Self>) -> Html {
        let make_table = |users: &Vec<User>| {
            html! {
                <div class="table-responsive">
                  <table class="table table-hover">
                    <thead>
                      <tr>
                        <th>{"User ID"}</th>
                        <th>{"Email"}</th>
                        <th>{"Display name"}</th>
                        <th>{"First name"}</th>
                        <th>{"Last name"}</th>
                        <th>{"Creation date"}</th>
                        <th>{"Delete"}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {users.iter().map(|u| self.view_user(ctx, u)).collect::<Vec<_>>()}
                    </tbody>
                  </table>
                </div>
            }
        };
        match &self.users {
            None => html! {{"Loading..."}},
            Some(users) => {
                if users.is_empty() && !self.search_query.is_empty() {
                    html! {
                        <div class="alert alert-info" role="alert">
                            <i class="bi-info-circle me-2"></i>
                            {"No users found matching your search criteria."}
                        </div>
                    }
                } else {
                    make_table(users)
                }
            },
        }
    }

    fn view_user(&self, ctx: &Context<Self>, user: &User) -> Html {
        let link = &ctx.link();
        html! {
          <tr key={user.id.clone()}>
              <td><Link to={AppRoute::UserDetails{user_id: user.id.clone()}}>{&user.id}</Link></td>
              <td>{&user.email}</td>
              <td>{&user.display_name}</td>
              <td>{&user.first_name}</td>
              <td>{&user.last_name}</td>
              <td>{&user.creation_date.naive_local().date()}</td>
              <td>
                <DeleteUser
                  username={user.id.clone()}
                  on_user_deleted={link.callback(Msg::OnUserDeleted)}
                  on_error={link.callback(Msg::OnError)}/>
              </td>
          </tr>
        }
    }

    fn view_errors(&self) -> Html {
        match &self.common.error {
            None => html! {},
            Some(e) => html! {<div>{"Error: "}{e.to_string()}</div>},
        }
    }
}
