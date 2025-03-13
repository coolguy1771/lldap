use crate::{
    components::select::{SearchableSelect, SelectOptionProps},
    infra::common_component::{CommonComponent, CommonComponentParts},
};
use anyhow::{Error, Result};
use graphql_client::GraphQLQuery;
use std::collections::HashSet;
use yew::prelude::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../schema.graphql",
    query_path = "queries/add_user_to_group.graphql",
    response_derives = "Debug",
    variables_derives = "Clone",
    custom_scalars_module = "crate::infra::graphql"
)]
pub struct AddUserToGroup;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../schema.graphql",
    query_path = "queries/list_users.graphql",
    response_derives = "Debug,Clone,PartialEq,Eq,Hash",
    variables_derives = "Clone",
    custom_scalars_module = "crate::infra::graphql"
)]
pub struct ListUserNames;
pub type User = list_user_names::ListUserNamesUsers;

pub struct AddGroupMemberComponent {
    common: CommonComponentParts<Self>,
    /// The list of existing users, initially not loaded.
    user_list: Option<Vec<User>>,
    /// The currently selected users.
    selected_users: Vec<User>,
    /// For tracking add user status
    current_add_index: usize,
}

pub enum Msg {
    UserListResponse(Result<list_user_names::ResponseData>),
    SubmitAddMembers,
    AddMemberResponse(Result<add_user_to_group::ResponseData>),
    SelectionChanged(Vec<SelectOptionProps>),
}

#[derive(yew::Properties, Clone, PartialEq)]
pub struct Props {
    pub group_id: i64,
    pub users: Vec<User>,
    pub on_user_added_to_group: Callback<User>,
    pub on_error: Callback<Error>,
}

impl CommonComponent<AddGroupMemberComponent> for AddGroupMemberComponent {
    fn handle_msg(
        &mut self,
        ctx: &Context<Self>,
        msg: <Self as Component>::Message,
    ) -> Result<bool> {
        match msg {
            Msg::UserListResponse(response) => {
                self.user_list = Some(response?.users);
            }
            Msg::SubmitAddMembers => return self.submit_add_members(ctx),
            Msg::AddMemberResponse(response) => {
                response?;
                // Adding the user to the group succeeded
                if self.current_add_index < self.selected_users.len() {
                    let user = self.selected_users[self.current_add_index].clone();
                    
                    // Notify about the added user
                    ctx.props().on_user_added_to_group.emit(user);
                    
                    // Increment index and continue adding users if there are more
                    self.current_add_index += 1;
                    if self.current_add_index < self.selected_users.len() {
                        return self.add_next_user(ctx);
                    }
                }
            }
            Msg::SelectionChanged(options) => {
                // Convert selection to User objects
                self.selected_users = options
                    .into_iter()
                    .map(|props| User {
                        id: props.value,
                        display_name: props.text,
                    })
                    .collect();
                return Ok(true);
            }
        }
        Ok(true)
    }

    fn mut_common(&mut self) -> &mut CommonComponentParts<Self> {
        &mut self.common
    }
}

impl AddGroupMemberComponent {
    fn get_user_list(&mut self, ctx: &Context<Self>) {
        self.common.call_graphql::<ListUserNames, _>(
            ctx,
            list_user_names::Variables { filters: None },
            Msg::UserListResponse,
            "Error trying to fetch user list",
        );
    }

    fn submit_add_members(&mut self, ctx: &Context<Self>) -> Result<bool> {
        if self.selected_users.is_empty() {
            return Ok(false);
        }
        
        // Reset the index counter
        self.current_add_index = 0;
        
        // Start adding the first user
        self.add_next_user(ctx)
    }
    
    fn add_next_user(&mut self, ctx: &Context<Self>) -> Result<bool> {
        if self.current_add_index >= self.selected_users.len() {
            return Ok(true);
        }
        
        let user_id = self.selected_users[self.current_add_index].id.clone();
        
        self.common.call_graphql::<AddUserToGroup, _>(
            ctx,
            add_user_to_group::Variables {
                user: user_id,
                group: ctx.props().group_id,
            },
            Msg::AddMemberResponse,
            "Error trying to initiate adding the user to a group",
        );
        Ok(true)
    }

    fn get_selectable_user_list(&self, ctx: &Context<Self>, user_list: &[User]) -> Vec<User> {
        let user_groups = ctx.props().users.iter().collect::<HashSet<_>>();
        user_list
            .iter()
            .filter(|u| !user_groups.contains(u))
            .map(Clone::clone)
            .collect()
    }
    
    fn get_selectable_options(&self, ctx: &Context<Self>, user_list: &[User]) -> Vec<SelectOptionProps> {
        self.get_selectable_user_list(ctx, user_list)
            .into_iter()
            .map(|user| {
                let name = if user.display_name.is_empty() {
                    user.id.clone()
                } else {
                    user.display_name.clone()
                };
                SelectOptionProps {
                    value: user.id,
                    text: name,
                }
            })
            .collect()
    }
}

impl Component for AddGroupMemberComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut res = Self {
            common: CommonComponentParts::<Self>::create(),
            user_list: None,
            selected_users: Vec::new(),
            current_add_index: 0,
        };
        res.get_user_list(ctx);
        res
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        CommonComponentParts::<Self>::update_and_report_error(
            self,
            ctx,
            msg,
            ctx.props().on_error.clone(),
        )
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        if let Some(user_list) = &self.user_list {
            let selectable_options = self.get_selectable_options(ctx, user_list);
            
            if selectable_options.is_empty() {
                html! {
                    <div class="alert alert-info">
                        {"All users are already members of this group."}
                    </div>
                }
            } else {
                html! {
                    <div class="row">
                        <div class="col-md-8">
                            <div class="card">
                                <div class="card-body">
                                    <h5 class="card-title">{"Add Members"}</h5>
                                    
                                    <SearchableSelect 
                                        options={selectable_options}
                                        on_selection_change={link.callback(Msg::SelectionChanged)}
                                        multiple={true}
                                        placeholder={"Search for users..."}
                                    />
                                    
                                    <div class="mt-3">
                                        <button
                                            class="btn btn-primary"
                                            disabled={self.selected_users.is_empty() || self.common.is_task_running()}
                                            onclick={link.callback(|_| Msg::SubmitAddMembers)}>
                                            <i class="bi-person-plus me-2"></i>
                                            {
                                                if self.selected_users.len() > 1 {
                                                    format!("Add {} members", self.selected_users.len())
                                                } else {
                                                    "Add member".to_string()
                                                }
                                            }
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            }
        } else {
            html! {
                <div class="spinner-border text-primary" role="status">
                    <span class="visually-hidden">{"Loading users..."}</span>
                </div>
            }
        }
    }
}
