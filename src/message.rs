use mediatype::{media_type, names::*};
use std::str;
use teloxide::RequestError;
use teloxide::{
    prelude2::*,
    types::{InputFile, ParseMode},
};
use yozuk_sdk::prelude::*;

const MAX_TEXT_LENGTH: usize = 2048;

pub async fn render_output(
    bot: AutoSend<Bot>,
    msg: &Message,
    output: Output,
) -> Result<(), RequestError> {
    for block in output.blocks {
        render_block(bot.clone(), msg, block).await?;
    }
    Ok(())
}

async fn render_block(bot: AutoSend<Bot>, msg: &Message, block: Block) -> Result<(), RequestError> {
    match block {
        Block::Comment(comment) => {
            bot.send_message(msg.chat.id, comment.text).send().await?;
        }
        Block::Data(data) => {
            render_data(bot, msg, data).await?;
        }
        Block::Spoiler(spiler) => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "{}: <spoiler>{}</spoiler>",
                    spiler.title,
                    spiler.data.unsecure()
                ),
            )
            .parse_mode(ParseMode::Html)
            .send()
            .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "[unimplemented]".to_string())
                .send()
                .await?;
        }
    }
    Ok(())
}

async fn render_data(
    bot: AutoSend<Bot>,
    msg: &Message,
    block: block::Data,
) -> Result<(), RequestError> {
    let essence = block.media_type.essence();
    let data = block.data.data().unwrap();
    let text = str::from_utf8(data).ok();

    match text {
        Some(text) if text.len() <= MAX_TEXT_LENGTH => {
            bot.send_message(msg.chat.id, format!("<pre>{}</pre>", text))
                .parse_mode(ParseMode::Html)
                .send()
                .await?;
        }
        _ if essence.ty == IMAGE => {
            bot.send_photo(msg.chat.id, InputFile::memory(data.to_vec()))
                .send()
                .await?;
        }
        _ if essence == media_type!(AUDIO / MPEG) || essence == media_type!(AUDIO / MP4) => {
            bot.send_audio(msg.chat.id, InputFile::memory(data.to_vec()))
                .send()
                .await?;
        }
        _ if essence == media_type!(VIDEO / MP4) => {
            bot.send_video(msg.chat.id, InputFile::memory(data.to_vec()))
                .send()
                .await?;
        }
        _ => {
            let ext = new_mime_guess::get_extensions(essence.ty.as_str(), essence.subty.as_str())
                .and_then(|list| list.first())
                .unwrap_or(&"bin");
            bot.send_document(
                msg.chat.id,
                InputFile::memory(data.to_vec()).file_name(format!("data.{}", ext)),
            )
            .send()
            .await?;
        }
    }

    Ok(())
}
