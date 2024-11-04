use std::sync::{Arc, Mutex};

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        CreateChatCompletionRequest,
    },
    Client as OpenAIClient,
};
use futures::TryStreamExt;
use serenity::all::EditMessage;
use tiktoken_rs::{get_chat_completion_max_tokens, ChatCompletionRequestMessage as TikChatMsg};

const SYSTEM_MESSAGE: &str = r#"
You are a helpful and polite assistant in the DeskThing Discord server. You can answer questions about the DeskThing project, or just chat about anything.

Respond concisely and in a friendly tone, but keep responses clear and readable. Use bullet points or short paragraphs if needed. Maximum response length is 1500 characters.

If users request specific information, refer them to the following links if they are needed:
1. Website: <https://deskthing.app>
2. Discord: <https://deskthing.app/discord>
3. Reddit: <https://www.reddit.com/r/DeskThing/>
4. Trello: <https://trello.com/b/6v0paxqV/deskthing>
5. GitHub Repository: <https://github.com/ItsRiprod/DeskThing>
6. BuyMeACoffee: <https://buymeacoffee.com/riprod>
7. YouTube Channel: <https://www.youtube.com/@deskthing>
8. Twitter: <https://x.com/TheDeskThing>
9. Bluesky (Twitter replacement): <https://bsky.app/profile/deskthing.app>
10. App Template: <https://github.com/ItsRiprod/deskthing-template>
11. Server Source Code: <https://github.com/ItsRiprod/DeskThing>
12. Client Source Code: <https://github.com/ItsRiprod/deskthing-client>
13. Example Apps: <https://github.com/ItsRiprod/deskthing-apps>
14. App downloads: <https://deskthing.app/applications>
15. CarThing Hacking server (Thing Labs): <https://tl.mt/d>

**About DeskThing and CarThing**
- DeskThing helps extend the life of the discontinued CarThing product (2024). CarThing was a second-monitor smartphone device created by Spotify, primarily used with Spotify itself.
- CarThing was released in 2022, but it was unpopular with users and has since been discontinued. Spotify will end support for it on December 9, 2024, leaving many devices unused.
- DeskThing’s goal is to repurpose CarThing to increase productivity by removing the need for Bluetooth, adding local audio support, providing weather updates, and making it adaptable to various devices like Raspberry Pi, Android, and desktops.

**DeskThing Compatibility and Hosting**
- DeskThing works on any device with a web browser, even low-end devices.
- DeskThing servers can run on Windows, Linux, and macOS.

- DeskThing is designed around apps. Apps are also used to interact with music services, like Spotify. To configure some apps, click the settings icon in the app's bar.
- To install an app, go to Downloads -> App and click the 'download' icon next to the app name.
- To add an app repo, go in settings and add the repo URL under 'Saved App Repos'.
- To add apps via Zip file, go to Downloads -> Apps -> Upload app.
- The list of apps in the default, included repo is Discord, Image, MediaWin, Record, Spotify, System, Weather, and WeatherWave.
- The current list of third-party apps is in the <#1292217043881299999> channel.

- To install the server, download the latest release from <https://github.com/ItsRiprod/DeskThing/releases/latest> and follow the instructions in the README.
- The server should guide the user on how to connect to the client.
- The client is designed for the Spotify CarThing but also works with other devices with web browsers.

- For questions related to Superbird setup, including issues with pyamlboot, refer users to <https://github.com/bishopdynamics/superbird-tool>. Mention they need a recent version of Python and Git. On Mac, install Python and libusb with Homebrew, or on Windows, use winget for Python and Git. Run `python -m pip install git+https://github.com/superna9999/pyamlboot` to install pyamlboot (sudo may be needed on Linux).
- If asked how to set up deskthing, refer users to the README in the <https://github.com/ItsRiprod/DeskThing> repository.
- Some things to check for if the CarThing is not working or responding: Is the screen on? Are you using a decent quality USB cable? Is ADB installed, and if it is, are you using the server's built-in ADB client? If you're on mac, have you made the adb binary executable `chmod +x /Applications/DeskThing.app/Contents/Resources/mac/adb` and do you have the custom 8.4.4_adb_enabled-new.tar.xz image installed? Have you tried restarting the Car Thing?

**Guidelines:**
- Include the relevant link in your response when asked about a specific topic.
- If uncertain about a topic, politely suggest asking others in the server.
- If addressing a specific user, use their nickname in your reply.
- When mentioning links, wrap them in <> to prevent embeds from showing.
- Ideally, include two or less links in your response. It's okay to include more, if needed.
- Don't address 'the group' or 'those who might not know' in your response. Ideally, the response should be directed at a user.
- If you need to address a specific user, mention them via <@![the user's id]>. For example, if you want to address 'Riprod (276531165878288385)', you can say <@!276531165878288385>.
- You can refer to a user by their ID (<@![the user's id]>) or just by their nickname. Do not under any circumstances refer to the user by their username, or put an @ in front of their nickname.
- Only refer to previous responses if reasonable. A message may not necessarily be a response to the previous message.

- DO NOT HALLUCINATE.
- DO NOT MAKE UP FACTUAL INFORMATION.
"#;

async fn aoai_to_tiktoken(msg: ChatCompletionRequestMessage) -> TikChatMsg {
    match msg {
        ChatCompletionRequestMessage::System(msg) => TikChatMsg {
            role: "system".to_string(),
            content: match msg.content {
                ChatCompletionRequestSystemMessageContent::Text(text) => Some(text),
                ChatCompletionRequestSystemMessageContent::Array(_) => todo!(),
            },
            ..Default::default()
        },
        ChatCompletionRequestMessage::User(msg) => TikChatMsg {
            role: "user".to_string(),
            content: match msg.content {
                ChatCompletionRequestUserMessageContent::Text(text) => Some(text),
                ChatCompletionRequestUserMessageContent::Array(_) => todo!(),
            },
            ..Default::default()
        },
        ChatCompletionRequestMessage::Assistant(msg) => TikChatMsg {
            role: "assistant".to_string(),
            content: match msg.content {
                Some(text) => match text {
                    ChatCompletionRequestAssistantMessageContent::Text(text) => Some(text),
                    ChatCompletionRequestAssistantMessageContent::Array(_) => todo!(),
                },
                None => None,
            },
            ..Default::default()
        },
        ChatCompletionRequestMessage::Tool(_) => todo!(),
        ChatCompletionRequestMessage::Function(_) => todo!(),
    }
}
pub async fn process_message(
    msg: serenity::model::channel::Message,
    ctx: serenity::prelude::Context,
    openai_client: &OpenAIClient<OpenAIConfig>,
    ai_context: &Arc<Mutex<std::collections::HashMap<String, Vec<ChatCompletionRequestMessage>>>>,
) {
    const TOKEN_LIMIT: usize = 128000;
    const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);
    let ai_model: String =
        std::env::var("AI_MODEL").unwrap_or("llama-3.2-11b-vision-preview".to_string());

    let start_time = std::time::Instant::now();

    // Handle response streaming
    let typing = ctx.http.start_typing(msg.channel_id);

    let mut sent_msg = msg
        .reply(&ctx.http, "Generating response...")
        .await
        .expect("failed to send message");

    // Create user message once
    let user_message = ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(format!(
            "{} ({}): {}",
            msg.author_nick(&ctx.http).await.unwrap_or(msg.author.name),
            msg.author.id.get(),
            msg.content
        )),
        ..Default::default()
    });

    // Update context more efficiently
    let messages = {
        let mut context = ai_context.lock().unwrap();
        let channel_context = context.entry(msg.channel_id.to_string()).or_default();
        channel_context.push(user_message);
        channel_context.clone()
    };

    // Create system message once
    let sys_msg = ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text(SYSTEM_MESSAGE.to_string()),
        ..Default::default()
    });

    // Token counting and context building
    let mut final_messages = vec![];
    // get_chat_completion_max_tokens responds with the *remaining context length*
    let sys_tokens = TOKEN_LIMIT
        - get_chat_completion_max_tokens("o1-mini", &[aoai_to_tiktoken(sys_msg.clone()).await])
            .expect("failed to get token count");
    let mut current_tokens = sys_tokens;

    // Process messages in reverse order more efficiently
    for msg in messages.iter().rev() {
        let msg_tokens = TOKEN_LIMIT
            - get_chat_completion_max_tokens("o1-mini", &[aoai_to_tiktoken(msg.clone()).await])
                .expect("failed to get token count");
        if current_tokens + msg_tokens > TOKEN_LIMIT {
            break;
        }

        final_messages.push(msg.clone());
        current_tokens += msg_tokens;
    }

    final_messages.push(sys_msg);

    final_messages.reverse();

    // Create chat completion request
    let request = CreateChatCompletionRequest {
        model: ai_model,
        messages: final_messages,
        max_tokens: Some(1800),
        stream: Some(true),
        ..Default::default()
    };

    let prep_time = start_time.elapsed().as_secs_f64();

    let mut stream = openai_client
        .chat()
        .create_stream(request)
        .await
        .expect("failed to create stream");

    let mut response = String::with_capacity(2000); // Pre-allocate string capacity
    let mut last_update = std::time::Instant::now();

    while let Some(chunk) = stream.try_next().await.expect("failed to get next chunk") {
        if let Some(content) = chunk.choices[0].delta.content.clone() {
            response.push_str(&content);

            if last_update.elapsed() >= UPDATE_INTERVAL {
                last_update = std::time::Instant::now();
                let builder = EditMessage::new().content(&response).suppress_embeds(true);
                if let Err(e) = sent_msg.edit(&ctx.http, builder).await {
                    eprintln!("Failed to edit message: {}", e);
                }
            }
        }

        if chunk.choices[0].finish_reason.is_some() {
            let elapsed = start_time.elapsed().as_secs_f64();
            let final_response = format!(
                "{}\n-# Generated response in {:.3}s ({:.3}s prep). There may be [inaccuracies in AI output](<https://lib.guides.umd.edu/c.php?g=1340355&p=9880574>). Check important info.",
                response, elapsed - prep_time, prep_time
            );

            let builder = EditMessage::new()
                .content(&final_response)
                .suppress_embeds(true);
            if let Err(e) = sent_msg.edit(&ctx.http, builder).await {
                eprintln!("Failed to edit final message: {}", e);
            }
            // insert response into context
            let mut context = ai_context.lock().unwrap();
            let channel_context = context.entry(msg.channel_id.to_string()).or_default();
            channel_context.push(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(response),
                    ..Default::default()
                },
            ));
            break;
        }
    }

    typing.stop();
}
