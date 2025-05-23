use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                    .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message_content = input.value();
                    if message_content.trim().is_empty() { // Prevent sending empty messages
                        return false;
                    }
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(message_content),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default(); // Prevent potential form submission if input is in a form
            Msg::SubmitMessage
        });

        html! {
            <div class="flex w-screen">
                // Sidebar - Blue
                <div class="flex-none w-56 h-screen" style="background-color: #3498DB;"> // Blue background
                    <div class="text-xl p-3" style="color: white;">{"Users"}</div> // White text
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 rounded-lg p-2" style="background-color: #EBF5FB;"> // Light blue background for user item
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div style="color: #2C3E50; font-weight: bold;">{u.name.clone()}</div> // Dark blue/black text
                                        </div>
                                        <div class="text-xs" style="color: #E67E22;"> // Orange text for status/subtitle
                                            {"Online"} // Changed "Hi there!" to "Online" for context
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>

                // Main Content Area
                <div class="grow h-screen flex flex-col" style="background-color: #F4F6F6;"> // Light grey background for main chat area
                    // Chat Header - Blue background, Orange border
                    <div class="w-full h-14 flex items-center p-3" style="background-color: #3498DB; border-bottom: 3px solid #F39C12;"> // Blue background, Orange bottom border
                        <div class="text-xl" style="color: white;">{"💬 Chat!"}</div> // White text
                    </div>

                    // Messages Area - Light background, Orange border for consistency
                    <div class="w-full grow overflow-y-auto p-4" style="border-bottom: 2px solid #F39C12;"> // Added padding, overflow-y
                        {
                            self.messages.iter().map(|m| {
                                let user_profile = self.users.iter().find(|u| u.name == m.from);
                                let avatar_src = user_profile.map_or_else(
                                    || format!("https://avatars.dicebear.com/api/initials/{}.svg", m.from), // Fallback avatar
                                    |user| user.avatar.clone()
                                );

                                html!{
                                    // Message Bubble - Light Orange
                                    <div class="flex items-start mb-4"> // Changed items-end to items-start for typical chat layout
                                        <img class="w-10 h-10 rounded-full mr-3" src={avatar_src} alt="avatar"/>
                                        <div style="background-color: #FDEBD0; border-radius: 8px; padding: 10px; max-width: 70%;"> // Light orange background for message
                                            <div class="text-sm font-semibold" style="color: #D35400; margin-bottom: 4px;"> // Orange, slightly darker for sender name
                                                {m.from.clone()}
                                            </div>
                                            <div class="text-sm" style="color: #333333; word-wrap: break-word;"> // Dark grey/black text for message
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-2 rounded" src={m.message.clone()} style="max-width: 100%; height: auto;"/>
                                                } else {
                                                    {m.message.clone()}
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>

                    // Input Area - Light grey bar, Orange button
                    <div class="w-full h-16 flex px-3 items-center" style="background-color: #EAECEE; border-top: 1px solid #D5DBDB;"> // Light grey background, subtle top border
                        <input ref={self.chat_input.clone()} type="text" placeholder="Type a message..."
                               class="block w-full py-2 pl-4 pr-4 mx-2 rounded-full outline-none focus:border-blue-500" // Kept focus:border for visual cue if CSS is ever added
                               style="background-color: #FFFFFF; border: 1px solid #BCCCDC; color: #2C3E50; height: 40px;" // White input, light blue/grey border
                               name="message" required=true
                               onkeypress={ctx.link().batch_callback(|e: KeyboardEvent| {
                                   if e.key() == "Enter" {
                                       Some(Msg::SubmitMessage)
                                   } else {
                                       None
                                   }
                               })}
                        />
                        <button onclick={submit}
                                class="p-2 shadow-sm w-10 h-10 rounded-full flex justify-center items-center" // Adjusted padding
                                style="background-color: #F39C12; color: white; border: none; cursor: pointer;"> // Orange button
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" style="fill: white; width: 20px; height: 20px;"> // White icon, adjusted size
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}