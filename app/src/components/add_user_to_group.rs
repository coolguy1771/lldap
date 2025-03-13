use crate::{
    components::{
        select::{SearchableSelect, SelectOptionProps},
        user_details::Group,
    },
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
    query_path = "queries/get_group_list.graphql",
    response_derives = "Debug",
    variables_derives = "Clone",
    custom_scalars_module = "crate::infra::graphql"
)]
pub struct GetGroupList;
type GroupListGroup = get_group_list::GetGroupListGroups;

impl From<GroupListGroup> for Group {
    fn from(group: GroupListGroup) -> Self {
        Self {
            id: group.id,
            display_name: group.display_name,
        }
    }
}

pub struct AddUserToGroupComponent {
    common: CommonComponentParts<Self>,
    /// The list of existing groups, initially not loaded.
    group_list: Option<Vec<Group>>,
    /// The currently selected groups.
    selected_groups: Vec<Group>,
    /// For tracking add group status
    current_add_index: usize,
}

pub enum Msg {
    GroupListResponse(Result<get_group_list::ResponseData>),
    SubmitAddGroups,
    AddGroupResponse(Result<add_user_to_group::ResponseData>),
    SelectionChanged(Vec<SelectOptionProps>),
}

#[derive(yew::Properties, Clone, PartialEq)]
pub struct Props {
    pub username: String,
    pub groups: Vec<Group>,
    pub on_user_added_to_group: Callback<Group>,
    pub on_error: Callback<Error>,
}

impl CommonComponent<AddUserToGroupComponent> for AddUserToGroupComponent {
    fn handle_msg(
        &mut self,
        ctx: &Context<Self>,
        msg: <Self as Component>::Message,
    ) -> Result<bool> {
        match msg {
            Msg::GroupListResponse(response) => {
                self.group_list = Some(response?.groups.into_iter().map(Into::into).collect());
            }
            Msg::SubmitAddGroups => return self.submit_add_groups(ctx),
            Msg::AddGroupResponse(response) => {
                response?;
                // Adding the user to the group succeeded
                if self.current_add_index < self.selected_groups.len() {
                    let group = self.selected_groups[self.current_add_index].clone();
                    
                    // Notify about the added group
                    ctx.props().on_user_added_to_group.emit(group);
                    
                    // Increment index and continue adding groups if there are more
                    self.current_add_index += 1;
                    if self.current_add_index < self.selected_groups.len() {
                        return self.add_next_group(ctx);
                    }
                }
            }
            Msg::SelectionChanged(options) => {
                // Convert selection to Group objects
                self.selected_groups = options
                    .into_iter()
                    .map(|props| Group {
                        id: props.value.parse::<i64>().unwrap(),
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

impl AddUserToGroupComponent {
    fn get_group_list(&mut self, ctx: &Context<Self>) {
        self.common.call_graphql::<GetGroupList, _>(
            ctx,
            get_group_list::Variables,
            Msg::GroupListResponse,
            "Error trying to fetch group list",
        );
    }

    fn submit_add_groups(&mut self, ctx: &Context<Self>) -> Result<bool> {
        if self.selected_groups.is_empty() {
            return Ok(false);
        }
        
        // Reset the index counter
        self.current_add_index = 0;
        
        // Start adding the first group
        self.add_next_group(ctx)
    }
    
    fn add_next_group(&mut self, ctx: &Context<Self>) -> Result<bool> {
        if self.current_add_index >= self.selected_groups.len() {
            return Ok(true);
        }
        
        let group_id = self.selected_groups[self.current_add_index].id;
        
        self.common.call_graphql::<AddUserToGroup, _>(
            ctx,
            add_user_to_group::Variables {
                user: ctx.props().username.clone(),
                group: group_id,
            },
            Msg::AddGroupResponse,
            "Error trying to initiate adding the user to a group",
        );
        Ok(true)
    }

    fn get_selectable_group_list(&self, props: &Props, group_list: &[Group]) -> Vec<Group> {
        let user_groups = props.groups.iter().collect::<HashSet<_>>();
        group_list
            .iter()
            .filter(|g| !user_groups.contains(g))
            .map(Clone::clone)
            .collect()
    }
    
    fn get_selectable_options(&self, props: &Props, group_list: &[Group]) -> Vec<SelectOptionProps> {
        self.get_selectable_group_list(props, group_list)
            .into_iter()
            .map(|group| SelectOptionProps {
                value: group.id.to_string(),
                text: group.display_name,
            })
            .collect()
    }
}

impl Component for AddUserToGroupComponent {
    type Message = Msg;
    type Properties = Props;
    fn create(ctx: &Context<Self>) -> Self {
        let mut res = Self {
            common: CommonComponentParts::<Self>::create(),
            group_list: None,
            selected_groups: Vec::new(),
            current_add_index: 0,
        };
        res.get_group_list(ctx);
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
        if let Some(group_list) = &self.group_list {
            let selectable_options = self.get_selectable_options(ctx.props(), group_list);
            
            if selectable_options.is_empty() {
                html! {
                    <div class="alert alert-info">
                        {"User is already a member of all available groups."}
                    </div>
                }
            } else {
                html! {
                    <div class="row">
                        <div class="col-md-8">
                            <div class="card">
                                <div class="card-body">
                                    <h5 class="card-title">{"Add to Groups"}</h5>
                                    
                                    <SearchableSelect 
                                        options={selectable_options}
                                        on_selection_change={link.callback(Msg::SelectionChanged)}
                                        multiple={true}
                                        placeholder={"Search for groups..."}
                                    />
                                    
                                    <div class="mt-3">
                                        <button
                                            class="btn btn-primary"
                                            disabled={self.selected_groups.is_empty() || self.common.is_task_running()}
                                            onclick={link.callback(|_| Msg::SubmitAddGroups)}>
                                            <i class="bi-person-plus me-2"></i>
                                            {
                                                if self.selected_groups.len() > 1 {
                                                    format!("Add to {} groups", self.selected_groups.len())
                                                } else {
                                                    "Add to group".to_string()
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
                    <span class="visually-hidden">{"Loading groups..."}</span>
                </div>
            }
        }
    }
}
