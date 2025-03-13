use std::collections::HashSet;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct Select {
    node_ref: NodeRef,
}

#[derive(yew::Properties, Clone, PartialEq, Debug)]
pub struct SelectProps {
    pub children: ChildrenWithProps<SelectOption>,
    pub on_selection_change: Callback<Option<SelectOptionProps>>,
    #[prop_or(false)]
    pub searchable: bool,
    #[prop_or(false)]
    pub multiple: bool,
}

pub enum SelectMsg {
    OnSelectChange,
}

impl Select {
    fn get_nth_child_props(&self, ctx: &Context<Self>, nth: i32) -> Option<SelectOptionProps> {
        if nth == -1 {
            return None;
        }
        ctx.props()
            .children
            .iter()
            .nth(nth as usize)
            .map(|child| (*child.props).clone())
    }

    fn send_selection_update(&self, ctx: &Context<Self>) {
        let select_node = self.node_ref.cast::<web_sys::HtmlSelectElement>().unwrap();
        ctx.props()
            .on_selection_change
            .emit(self.get_nth_child_props(ctx, select_node.selected_index()))
    }
}

impl Component for Select {
    type Message = SelectMsg;
    type Properties = SelectProps;
    fn create(_: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        self.send_selection_update(ctx);
    }

    fn update(&mut self, ctx: &Context<Self>, _: Self::Message) -> bool {
        self.send_selection_update(ctx);
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <select class="form-select"
              ref={self.node_ref.clone()}
              disabled={ctx.props().children.is_empty()}
              onchange={ctx.link().callback(|_| SelectMsg::OnSelectChange)}
              multiple={ctx.props().multiple}>
            { ctx.props().children.clone() }
            </select>
        }
    }
}

#[derive(yew::Properties, Clone, PartialEq, Eq, Debug)]
pub struct SelectOptionProps {
    pub value: String,
    pub text: String,
}

#[function_component(SelectOption)]
pub fn select_option(props: &SelectOptionProps) -> Html {
    html! {
      <option value={props.value.clone()}>
        {&props.text}
      </option>
    }
}

// A searchable select component that filters options as you type
pub struct SearchableSelect {
    search_ref: NodeRef,
    filtered_options: Vec<SelectOptionProps>,
    selected_options: HashSet<String>,
}

#[derive(yew::Properties, Clone, PartialEq, Debug)]
pub struct SearchableSelectProps {
    pub options: Vec<SelectOptionProps>,
    pub on_selection_change: Callback<Vec<SelectOptionProps>>,
    #[prop_or(false)]
    pub multiple: bool,
    #[prop_or("Search...".to_string())]
    pub placeholder: String,
}

pub enum SearchableSelectMsg {
    OnSearchChange,
    OnOptionSelect(SelectOptionProps, bool),
    OnSubmit,
}

impl Component for SearchableSelect {
    type Message = SearchableSelectMsg;
    type Properties = SearchableSelectProps;
    
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            search_ref: NodeRef::default(),
            filtered_options: ctx.props().options.clone(),
            selected_options: HashSet::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SearchableSelectMsg::OnSearchChange => {
                let search_input = self.search_ref.cast::<HtmlInputElement>().unwrap();
                let search_value = search_input.value().to_lowercase();
                
                if search_value.is_empty() {
                    self.filtered_options = ctx.props().options.clone();
                } else {
                    self.filtered_options = ctx.props().options
                        .iter()
                        .filter(|option| option.text.to_lowercase().contains(&search_value))
                        .cloned()
                        .collect();
                }
                true
            },
            SearchableSelectMsg::OnOptionSelect(option, selected) => {
                if ctx.props().multiple {
                    if selected {
                        self.selected_options.insert(option.value.clone());
                    } else {
                        self.selected_options.remove(&option.value);
                    }
                } else {
                    self.selected_options.clear();
                    if selected {
                        self.selected_options.insert(option.value.clone());
                    }
                    // For single select, immediately emit the selection
                    let selected = if selected {
                        vec![option]
                    } else {
                        vec![]
                    };
                    ctx.props().on_selection_change.emit(selected);
                }
                true
            },
            SearchableSelectMsg::OnSubmit => {
                if ctx.props().multiple {
                    // Get all selected options
                    let selected_options: Vec<SelectOptionProps> = ctx.props().options
                        .iter()
                        .filter(|option| self.selected_options.contains(&option.value))
                        .cloned()
                        .collect();
                    
                    ctx.props().on_selection_change.emit(selected_options);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        
        html! {
            <div class="searchable-select">
                <div class="form-group mb-2">
                    <input 
                        type="text" 
                        class="form-control"
                        ref={self.search_ref.clone()}
                        placeholder={ctx.props().placeholder.clone()}
                        oninput={link.callback(|_| SearchableSelectMsg::OnSearchChange)}
                    />
                </div>
                <div class="list-group mb-2" style="max-height: 200px; overflow-y: auto;">
                    {
                        self.filtered_options.iter().map(|option| {
                            let is_selected = self.selected_options.contains(&option.value);
                            let option_clone = option.clone();
                            html! {
                                <div 
                                    class={classes!("list-group-item", "list-group-item-action", if is_selected { "active" } else { "" })}
                                    onclick={link.callback(move |_| SearchableSelectMsg::OnOptionSelect(option_clone.clone(), !is_selected))}
                                >
                                    {&option.text}
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                {
                    if ctx.props().multiple {
                        html! {
                            <button 
                                class="btn btn-primary" 
                                onclick={link.callback(|_| SearchableSelectMsg::OnSubmit)}
                                disabled={self.selected_options.is_empty()}
                            >
                                {"Add Selected"}
                            </button>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }
}
