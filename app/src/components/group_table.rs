use crate::{
    components::{
        delete_group::DeleteGroup,
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
    query_path = "queries/get_group_list.graphql",
    response_derives = "Debug,Clone,PartialEq,Eq",
    custom_scalars_module = "crate::infra::graphql"
)]
pub struct GetGroupList;

use get_group_list::ResponseData;

pub type Group = get_group_list::GetGroupListGroups;

pub struct GroupTable {
    common: CommonComponentParts<Self>,
    groups: Option<Vec<Group>>,
    search_query: String,
}

pub enum Msg {
    ListGroupsResponse(Result<ResponseData>),
    OnGroupDeleted(i64),
    OnError(Error),
    OnSearchChange(String),
    SearchGroups,
}

impl CommonComponent<GroupTable> for GroupTable {
    fn handle_msg(&mut self, ctx: &Context<Self>, msg: <Self as Component>::Message) -> Result<bool> {
        match msg {
            Msg::ListGroupsResponse(groups) => {
                self.groups = Some(groups?.groups.into_iter().collect());
                Ok(true)
            }
            Msg::OnError(e) => Err(e),
            Msg::OnGroupDeleted(group_id) => {
                debug_assert!(self.groups.is_some());
                self.groups.as_mut().unwrap().retain(|u| u.id != group_id);
                Ok(true)
            }
            Msg::OnSearchChange(query) => {
                self.search_query = query;
                Ok(false)
            }
            Msg::SearchGroups => {
                // Since we don't have a RequestFilter for groups in the current API
                // We'll get all groups and filter them client-side
                self.common.call_graphql::<GetGroupList, _>(
                    ctx,
                    get_group_list::Variables {},
                    Msg::ListGroupsResponse,
                    "Error trying to fetch groups",
                );
                Ok(true)
            }
        }
    }

    fn mut_common(&mut self) -> &mut CommonComponentParts<Self> {
        &mut self.common
    }
}

impl Component for GroupTable {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut table = GroupTable {
            common: CommonComponentParts::<Self>::create(),
            groups: None,
            search_query: String::new(),
        };
        table.common.call_graphql::<GetGroupList, _>(
            ctx,
            get_group_list::Variables {},
            Msg::ListGroupsResponse,
            "Error trying to fetch groups",
        );
        table
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        CommonComponentParts::<Self>::update(self, ctx, msg)
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
              {self.view_search_bar(ctx)}
              {self.view_groups(ctx)}
              {self.view_errors()}
            </div>
        }
    }
}

impl GroupTable {
    fn view_search_bar(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
            <div class="card mb-3">
                <div class="card-body">
                    <h5 class="card-title">{"Search Groups"}</h5>
                    <div class="row g-3 align-items-center">
                        <div class="col-auto">
                            <input 
                                type="text" 
                                class="form-control" 
                                placeholder="Search by group name"
                                value={self.search_query.clone()}
                                oninput={link.callback(|e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    Msg::OnSearchChange(input.value())
                                })}
                                onkeypress={link.callback(|e: KeyboardEvent| {
                                    if e.key() == "Enter" {
                                        Msg::SearchGroups
                                    } else {
                                        Msg::OnSearchChange("".to_string())
                                    }
                                })}
                            />
                        </div>
                        <div class="col-auto">
                            <button 
                                class="btn btn-primary"
                                onclick={link.callback(|_| Msg::SearchGroups)}
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
                                                Msg::SearchGroups
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

    fn view_groups(&self, ctx: &Context<Self>) -> Html {
        let make_table = |groups: &Vec<Group>| {
            let filtered_groups = if self.search_query.is_empty() {
                groups.clone()
            } else {
                // Filter groups client-side by display_name
                groups
                    .iter()
                    .filter(|g| g.display_name.to_lowercase().contains(&self.search_query.to_lowercase()))
                    .cloned()
                    .collect()
            };

            if filtered_groups.is_empty() && !self.search_query.is_empty() {
                html! {
                    <div class="alert alert-info" role="alert">
                        <i class="bi-info-circle me-2"></i>
                        {"No groups found matching your search criteria."}
                    </div>
                }
            } else {
                html! {
                    <div class="table-responsive">
                      <table class="table table-hover">
                        <thead>
                          <tr>
                            <th>{"Group name"}</th>
                            <th>{"Creation date"}</th>
                            <th>{"Delete"}</th>
                          </tr>
                        </thead>
                        <tbody>
                          {filtered_groups.iter().map(|u| self.view_group(ctx, u)).collect::<Vec<_>>()}
                        </tbody>
                      </table>
                    </div>
                }
            }
        };
        match &self.groups {
            None => html! {{"Loading..."}},
            Some(groups) => make_table(groups),
        }
    }

    fn view_group(&self, ctx: &Context<Self>, group: &Group) -> Html {
        let link = ctx.link();
        html! {
          <tr key={group.id}>
              <td>
                <Link to={AppRoute::GroupDetails{group_id: group.id}}>
                  {&group.display_name}
                </Link>
              </td>
              <td>
                {&group.creation_date.naive_local().date()}
              </td>
              <td>
                <DeleteGroup
                  group={group.clone()}
                  on_group_deleted={link.callback(Msg::OnGroupDeleted)}
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
