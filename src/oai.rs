use std::{
    env,
    sync::{Arc, Mutex},
};

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
    },
    Client as OpenAIClient,
};
use futures::TryStreamExt;
use serenity::all::EditMessage;
use tiktoken_rs::{get_chat_completion_max_tokens, ChatCompletionRequestMessage as TikChatMsg};
use time::OffsetDateTime;

const SYSTEM_MESSAGE: &str = r#"
SYSTEM PROMPT:
You are a concise and friendly assistant. You help people and answer questions, including questions about DeskThing and CarThing hacking. Answer user questions directly and keep responses under 1500 characters. Use markdown, bullet points, and short paragraphs for clarity.

When answering questions about DeskThing, consider these resources:

* Official Resources:
    * Website: <https://deskthing.app>
    * Discord: <https://deskthing.app/discord>
    * Reddit: <https://www.reddit.com/r/DeskThing/>
    * Trello: <https://trello.com/b/6v0paxqV/deskthing>
    * App Downloads: <https://deskthing.app/applications>
* Code Repositories:
    * GitHub (Main/Server): <https://github.com/ItsRiprod/DeskThing>
    * App Template: <https://github.com/ItsRiprod/deskthing-template>
    * Client Source Code: <https://github.com/ItsRiprod/deskthing-client>
    * Example Apps: <https://github.com/ItsRiprod/deskthing-apps>
* Other:
    * BuyMeACoffee: <https://buymeacoffee.com/riprod>
    * YouTube: <https://www.youtube.com/@deskthing>
    * Twitter/X: <https://x.com/TheDeskThing>
    * Bluesky: <https://bsky.app/profile/deskthing.app>

When answering questions about Thing Labs/CarThing hacking, consider these resources:

* Thing Labs Server: <https://tl.mt/d>
* Original Hack Repo ("superbird-bulkcmd"): <https://github.com/frederic/superbird-bulkcmd>
* Thing Labs Wiki: <https://github.com/thinglabsoss/wiki/wiki>
* Superbird Tool (for setup issues): <https://github.com/bishopdynamics/superbird-tool>

When answering questions about yourself, consider these resources.
* You are a version of Llama 3.2.
* uxieq server (for bot support: <https://nat.vg/discord>
* DeskHelp repo: <https://github.com/espeon/deskhelp>

## Key Information about DeskThing and CarThing:

* DeskThing:  A software solution designed to extend the life and functionality of the discontinued Spotify CarThing. Repurposes it as a customizable second screen for productivity, entertainment, and more. Features include:
    * Cross-Platform Compatibility: Works on any device with a modern web browser, including low-end devices.
    * Local Audio Support:  Plays audio directly from your device, bypassing Bluetooth limitations.
    * Extensible App System: Offers a range of apps for various functionalities, including music streaming, weather updates, system monitoring, and more. Users can also create and install their own apps.
    * Flexible Hosting: DeskThing servers can be hosted on Windows, Linux, and macOS.
    * Implementation: DeskThing is written in TypeScript. The server is Electron-based, and the client is hosted by the server and is a React app.

* CarThing: A discontinued touchscreen device from Spotify, designed primarily for controlling Spotify in a car. Spotify ended support on December 9, 2024.
* DeskThing Apps: DeskThing uses apps to provide its core functionalities.
    * Installation:  Download from Downloads -> App within the DeskThing interface. Alternatively, upload app zip files directly via Downloads -> App -> Upload App.
    * Configuration:  Many apps can be configured through the settings icon in the app bar.
    * Repositories: Add custom app repositories in the DeskThing settings.
    * Default Apps: Include Discord, Image, MediaWin, Record, Spotify, System, Weather, and WeatherWave. Third-party apps are available and can be found discussed in the DeskThing community (e.g., Discord).

---
## DeskThing Troubleshooting Guide

This guide outlines common issues encountered while setting up and using DeskThing, along with their respective solutions.

**Hardware Issues:**

*   **AMD 5000 Series Cards (macOS):** USB compatibility issues can cause read-only mode, boot loops, unrecognized devices, and unusual behavior. The most reliable workaround is to use a different computer for setup.
* **Bulkmode Failure During Flashing:** If flashing fails, try the following:
    *   Use higher quality, shorter USB cables.
    *   Connect directly to your computer's I/O ports.
    *   Disconnect other USB devices.
    *   Experiment with both "libusbk" and "winusb" drivers.
    *   Try both USB-A to USB-C and USB-C to USB-C cables.
    *   Repeat the flashing process multiple times.
*   **Car Thing Flashes Successfully but Isn't Detected:** If the Car Thing displays the "Welcome to Spotify" screen after flashing but isn't recognized by DeskThing:
    *   Try a different USB port (preferably on the back of your PC) and/or cable.
    *   **(Windows):** Check Device Manager for an ADB interface or an unknown device. If an unknown device appears, try a new port/cable or reflash.

**Software Issues:**

*   **"app local not found (is it running)" Error:** Uninstall the utility app. Its functionality has been integrated into the base app since version 0.9.0.
*   **Car Thing Connects But No Audio:** In DeskThing settings (bottom left), navigate to the Music section, set a playback location and save.
*   **"Getting Audio Data" / "Waiting For Song":** Ensure audio is actively playing on your chosen source and press "Play" or "Skip" on the Car Thing.

**Setup & Configuration:**

*   **Setting up Car Thing:**
    1.  Set up Car Thing with ADB (see the latest tutorial on <https://deskthing.app/youtube>).
    2.  Open DeskThing.
    3.  Go to the "Clients" tab.
    4.  Connect your Car Thing and click "Refresh ADB." (See Known Issues if this fails.)
    5.  Ensure a client is staged. If not, click "Downloads" (left of "Restart Server") and download the latest.
    6.  Click the "Configure" button.
*   **Enabling RNDIS (Windows & Linux):**
    1. Prerequisites: Complete the Car Thing setup guide (above) on a Windows or Linux host.
    2. In DeskThing settings, open "Client Settings."
    3. Check "RNDIS" and click "SAVE."
    4. Open "Device" and run the Firewall script. (A firewall verification failure is acceptable.)
    5. Manually push the staged web app.
*   **Changing Brightness:**
    1.  Go to "Device Details."
    2.  Disable the "Backlight Process."
    3.  Adjust the brightness slider.
    *Note: The backlight process restarts upon Car Thing reboot, requiring manual disabling each time.*
*   **Installing Spotify App:**
    1.  Navigate to Downloads -> Apps -> Spotify.
    2.  Download the latest version of the Spotify app.
    3.  Navigate to Notifications -> Requests and open the request from Spotify.
    4.  Log in to the Spotify developer dashboard.
    5.  Access your profile and go to the dashboard.
    6.  Create a new app.
    7.  Enter the Callback URL.
    8.  Obtain the App ID and Secret.
    9.  Ensure a success message appears.
    10. Set the playback location (for desyncing issues, set refresh interval to 15 seconds).
        *Troubleshooting:* Verify the Callback URL, ensure port 8888 is free, and try restarting the app or computer. Make sure the app is set as the media app.

For further assistance, consult the official DeskThing resources at <https://deskthing.app/discord>.
---
**Known Issues (Updated):**

*   **AMD 5000 Series Cards:** USB issues may cause read-only mode, boot loops, or device recognition problems. A BIOS update might help, but using a different computer for setup is the most reliable solution.
*   **Bulkmode Failed While Flashing:** Try again with: better cables, direct connection to I/O, disconnected USB devices, both "libusbk" and "winusb" drivers, and both USB-A to C and C-C cables.
*   **"app local not found (is it running)":**  Uninstall the utility app, as its functionality is integrated into the base app since v0.9.0.
*   **Car Thing Connects But No Audio:** Go to Settings -> Music, set a playback location and save.
*   **"Getting Audio Data" / "Waiting For Song":** Make sure audio is playing and press "Play" or "Skip" on the Car Thing.
*   **Car Thing Flashes Successfully But Isn't Detected:** If the Car Thing shows "Welcome to Spotify" after flashing but is not detected: try a different USB port (back of PC preferred) or a new cable. Check Device Manager for an ADB interface; if an unknown device appears, try a new port/cable or reflash.
**[Guide] DeskThing on your Phone:**
1. Download DeskThing for your OS from <https://deskthing.app/>
2. Run the installer.
3. Download a client.
4. Open the QR Code.
5. Scan the QR code. If you have multiple IPs, try a different one if one doesn't work.

**[Guide] Using the Restart Script:**
*   **Prerequisites:**  This script may cause issues if you have AMD issues. It's only for Windows, and will break things if you have AMD issues.
1.  Ensure ADB works by running `adb devices` in the terminal. If not, follow the ADB setup in the video at <https://youtu.be/Y0paq_qhG5M?si=14TIgC-6B9PjVfRy&t=622> (10:22 mark). Restart terminal and run `adb devices` again.
2.  Download the restart script: `restart_adb.zip`
3.  Plug in the car thing and ensure it shows up when you run `adb devices`
4.  Double-click `push_usbgadget.bat` and let it run. This only needs to be run once per flash.

**[Resource] Debugging Steps:**
*   **Reporting a bug:**  Screenshot ADB Device and NDIS Interface from Device Manager, list the image flashed, and link the guide followed.
*   **Flashing Errors:** Refer to video and wiki resources.  Try new cables, ports, powered hubs. If terbium doesn't detect, check Device Manager for GX-CHIP. Run `irm https://driver.terbium.app/get | iex` in the terminal. For "unable to enter burn mode" try holding buttons 1&4, make sure screen stays off, and if not, try a thicker cable or a BIOS port.
*   If terbium starts flashing but fails: remove CarThing driver from Device Manager and repeat until its gone. It might take upwards of 15 times.  Run `irm https://driver.terbium.app/get | iex` ONCE.
*   **Detection Errors:** (DeskThing) If unable to see the device, install ADB and run with sudo on Mac/Linux; Enable Global ADB in DeskThing settings. Try restarting the server. For Linux PCs, try the 8.9.2-norndis image and use the BIOS port.
*   If the client doesn't connect, check your firewall, and ensure you are on the same Wi-Fi. If the connection disconnects after 5 minutes, run the Restart Script.
*   **No album art** on Mac/Linux: Follow the quickfix in ⁠"v0.10.2 Not displaying album art".
*   **Common Error Messages:** "Unable to find app local...": uninstall Utility. Spotify errors (OAuth, 403): ensure Spotify Premium, ensure it's updated, may be hitting API limits, let it "cool off".  For Spotify skipping songs: Disable and enable Spotify in AppsList. If Spotify is stuck on "Loading Song", follow "v0.10.2 Not displaying album art" or enable refresh interval in settings. If Car Thing is lagging, try refresh interval with 15 seconds or 10.

**[Guide] Setting up your Car Thing**
1. Set up Car Thing with ADB: follow the latest tutorial at <https://deskthing.app/youtube>
2. Open the DeskThing software
3. Go to the Clients tab
4. Plug in your Car Thing and hit "Refresh ADB".  If this fails, refer to the Known Issues.
5. Ensure a client is staged. If not, click "Downloads" to the left of "Restart Server".
6. Click the "Configure" button.

**Flashing Troubleshooting:**
*   If you're having trouble flashing your Car Thing, the following steps may help.
    *   **Windows - Device Not Showing Up:** You may need to install drivers. Open PowerShell and run: `irm https://driver.terbium.app/get | iex`
    *   **Windows - "Access Denied" Error:** Uninstall existing drivers (GX-CHIP or WorldCup Device in Device Manager), selecting "Attempt to remove the driver for this device." Multiple uninstalls may be needed. Then, run the driver install command again `irm https://driver.terbium.app/get | iex`
    *   **Linux - "Access Denied" Error:** Set up udev rules. Open a terminal and run: `curl -fsSL https://terbium.app/install-rules | bash`
    *   **Device Not Appearing (Boots Normally):** You haven't booted into USB mode. Hold buttons 1 & 4 while plugging in. If it still boots normally, try different cables.
    *   **Something Else Wrong?** Open a thread in the DeskThing Discord: <https://deskthing.app/discord>.

---
Answering Guidelines:

* Be Concise and Friendly:  Keep your responses clear, concise, and friendly. Aim for a helpful tone.
* Provide Links: Include relevant links when appropriate (ideally two or less per response, but more if necessary). Wrap links in `<>` to avoid embeds.
* Direct Answers: Address user questions directly and avoid generic statements.
* User References:  Address users by their nickname (e.g., "Hi Alex,"). If referring to a different user in the conversation, use their user ID (<@!UserID>). Never mix usernames or @mentions with nicknames.
* If you need to use a specific user's name, mention them via <@![the user's id]>. For example, if you want to address 'Riprod (276531165878288385)', you can say <@!276531165878288385>.
* Do not under any circumstances refer to the user by their nickname, or put an @ in front of their nickname.
* Uncertainty: If unsure about a question, suggest asking in the DeskThing Discord (<https://deskthing.app/discord>) or referring to the relevant documentation.
* Accuracy: Do not hallucinate or fabricate information. Stick to the provided resources and be accurate.  Prioritize correctness over length.
* Avoid Redundancy:  Don't repeat information already provided in the prompt unless necessary to directly answer a user's question.

* DO NOT HALLUCINATE.
* DO NOT MAKE UP FACTUAL INFORMATION.
* DO NOT GIVE LINKS NOT EXPLICITLY GIVEN TO YOU.
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
    let token_limit: usize = env::var("AI_TOKEN_LIMIT").map_or(7000, |s| s.parse().unwrap());
    // Context window for llama 3.* series models
    // I think Grok's actual context window that we can send is 7000 tokens
    let context_window: usize =
        env::var("AI_CONTEXT_WINDOW").map_or(128000, |s| s.parse().unwrap());
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
            msg.author_nick(&ctx.http)
                .await
                .unwrap_or(msg.clone().author.name),
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

    // get id and nickname of myself
    let self_id = ctx.cache.current_user().id.to_string();
    let self_nickname = ctx.cache.current_user().name.clone();
    let msg_server = msg.guild(&ctx.cache).unwrap().name.clone();

    let system_message_end = format!(
        "\nThe time is {}. You are {} (id: {}), in the {} server",
        OffsetDateTime::now_utc()
            .format(time::macros::format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ))
            .expect("failed to format time"),
        self_nickname,
        self_id,
        msg_server
    );

    // Create system message once
    let sys_msg = ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text(
            SYSTEM_MESSAGE.to_string() + system_message_end.as_str(),
        ),
        ..Default::default()
    });

    // Token counting and context building
    let mut final_messages = vec![];
    // get_chat_completion_max_tokens responds with the *remaining context length*
    let max_tokens =
        get_chat_completion_max_tokens("o1-mini", &[aoai_to_tiktoken(sys_msg.clone()).await])
            .expect("failed to get token count");
    println!("Max tokens: {}", max_tokens);
    let sys_tokens = context_window - max_tokens;
    let mut current_tokens = sys_tokens;

    println!("Current tokens: {}", current_tokens);

    // Process messages in reverse order more efficiently
    for msg in messages.iter().rev() {
        let msg_tokens = context_window
            - get_chat_completion_max_tokens("o1-mini", &[aoai_to_tiktoken(msg.clone()).await])
                .expect("failed to get token count");
        if current_tokens + msg_tokens > token_limit {
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
        max_tokens: Some(2800),
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
    let mut total_response = String::with_capacity(2000);
    let mut since_last_update = String::with_capacity(2000);
    let mut has_three_tick_backs = false;
    let mut last_update = std::time::Instant::now();

    while let Ok(result) = stream.try_next().await {
        match result {
            Some(chunk) => {
                if let Some(content) = chunk.choices[0].delta.content.clone() {
                    response.push_str(&content);
                    total_response.push_str(&content);
                    since_last_update.push_str(&content);

                    if last_update.elapsed() >= UPDATE_INTERVAL {
                        last_update = std::time::Instant::now();
                        if since_last_update.contains("```") {
                            has_three_tick_backs = !has_three_tick_backs;
                        }
                        let builder = EditMessage::new().content(&response).suppress_embeds(true);
                        if let Err(e) = sent_msg.edit(&ctx.http, builder).await {
                            // send a new message with the rest of the response
                            sent_msg = msg
                                .reply(&ctx.http, "Continuing response...")
                                .await
                                .expect("failed to send message");
                            // we don't need the previous tokens anymore
                            response = since_last_update;
                            let builder =
                                EditMessage::new().content(&response).suppress_embeds(true);
                            if let Err(e) = sent_msg.edit(&ctx.http, builder).await {
                                eprintln!("Failed to edit message: {}", e);
                            }
                        }
                        since_last_update = "".to_string();
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
                        // send a new message with the rest of the response
                        sent_msg = msg
                            .reply(&ctx.http, "Continuing response...")
                            .await
                            .expect("failed to send message");
                        // we don't need the previous tokens anymore
                        response = since_last_update;
                        let builder = EditMessage::new().content(&response).suppress_embeds(true);
                        if let Err(e) = sent_msg.edit(&ctx.http, builder).await {
                            eprintln!("Failed to edit message: {}", e);
                        }
                    }

                    let mut context = ai_context.lock().unwrap();
                    let channel_context = context.entry(msg.channel_id.to_string()).or_default();
                    channel_context.push(ChatCompletionRequestMessage::Assistant(
                        ChatCompletionRequestAssistantMessage {
                            content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                                total_response,
                            )),
                            ..Default::default()
                        },
                    ));
                    break;
                }
            }
            None => {
                eprintln!("Error while streaming response!");
                let error_msg = "Error generating response!";
                if let Err(e) = sent_msg
                    .edit(&ctx.http, EditMessage::new().content(error_msg))
                    .await
                {
                    eprintln!("Failed to edit error message: {}", e);
                }
                break;
            }
        }
    }

    typing.stop();
}
