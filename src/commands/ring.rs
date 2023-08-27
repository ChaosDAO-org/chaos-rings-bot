use std::{env, fmt};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Cursor;
use std::path::Path;
use anyhow::Context;

use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageOutputFormat, ImageResult, Rgba, RgbaImage};
use image::imageops::{FilterType, overlay};
use image::io::Reader as ImageReader;
use serenity::builder::CreateApplicationCommand;
use serenity::model::guild::Member;
use serenity::model::prelude::{Attachment, AttachmentType, RoleId};
use serenity::model::prelude::command::CommandOptionType;

#[derive(Debug)]
pub enum DaoRole {
    Frens,
    Regulars,
    DAOists,
}

#[derive(Debug)]
pub struct UserRecoverableError {
    reason: String,
}


impl Display for UserRecoverableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error while preparing an avatar: {}", self.reason)
    }
}

impl Error for UserRecoverableError {}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("ring")
        .description("Overlay a ChaosDAO ring to an avatar")
        .create_option(
            |option| {
                option
                    .name("avatar")
                    .description("A square profile picture")
                    .kind(CommandOptionType::Attachment)
                    .required(true)
            },
        )
}

pub async fn run<'a>(user: &'a Member, user_image: &'a Attachment) -> anyhow::Result<AttachmentType<'a>> {
    let ring_path = match find_dao_role(user)? {
        DaoRole::Frens => { load_env_var("CHAOSRING_FRENS") }
        DaoRole::Regulars => { load_env_var("CHAOSRING_REGULARS") }
        DaoRole::DAOists => { load_env_var("CHAOSRING_DAOISTS") }
    }?;

    let ring = ImageReader::open(Path::new(&ring_path))?
        .decode()?;

    let avatar = user_image.download().await?;
    let avatar = image::load_from_memory(&avatar)
        .and_then(|avatar| overlay_ring(&avatar.to_rgba8(), &ring.to_rgba8()))?;

    let buf: Vec<u8> = Vec::with_capacity(avatar.as_raw().len());
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(buf);
    avatar.write_to(&mut cursor, ImageOutputFormat::Png)?;
    let attachment = AttachmentType::Bytes {
        data: Cow::from(cursor.into_inner()),
        filename: String::from("avatar.png"),
    };

    Ok(attachment)
}

fn load_env_var(variable: &str) -> anyhow::Result<String> {
    let var = env::var(variable)
        .with_context(|| format!("No variable with name {} found in the environment", &variable))?;
    Ok(var)
}

fn parse_role_id(value: String) -> anyhow::Result<u64> {
    let value = value.parse::<u64>()?;
    Ok(value)
}

fn find_dao_role(member: &Member) -> anyhow::Result<DaoRole> {
    let user_roles: &Vec<RoleId> = &member.roles;

    let role_fren_id = load_env_var("DAO_ROLE_FREN")
        .and_then(parse_role_id)?;
    let role_regular_id = load_env_var("DAO_ROLE_REGULAR")
        .and_then(parse_role_id)?;
    let role_daoist_id = load_env_var("DAO_ROLE_DAOIST")
        .and_then(parse_role_id)?;

    let fren: RoleId = RoleId(role_fren_id);
    let regular: RoleId = RoleId(role_regular_id);
    let daoist: RoleId = RoleId(role_daoist_id);

    if user_roles.contains(&daoist) {
        Ok(DaoRole::DAOists)
    } else if user_roles.contains(&regular) {
        Ok(DaoRole::Regulars)
    } else if user_roles.contains(&fren) {
        Ok(DaoRole::Frens)
    } else {
        let inner = UserRecoverableError { reason: String::from("User is not a DAOist, regular or fren") };
        Err(anyhow::Error::new(inner))
    }
}

fn overlay_ring(avatar: &RgbaImage, ring: &RgbaImage) -> ImageResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    println!("dimensions: avatar @ {:?}, ring @ {:?}", avatar.dimensions(), ring.dimensions());

    let mut ring = DynamicImage::ImageRgba8(ring.clone());
    let avatar = DynamicImage::ImageRgba8(avatar.clone());
    // images must be square so one dimension is enough
    let avatar_side = avatar.width();
    let ring_side = ring.width();
    if ring_side > avatar_side {
        ring = ring.resize_to_fill(avatar_side, avatar_side, FilterType::Nearest);
    }
    let ring_side = ring.width();
    let circumference_width = get_ring_width(&ring);
    let scaled_avatar = avatar.resize_to_fill(ring_side - 2 * circumference_width,
                                              ring_side - 2 * circumference_width,
                                              FilterType::Nearest);

    let mut buffer = RgbaImage::new(ring_side, ring_side);
    buffer.copy_from(&scaled_avatar, circumference_width, circumference_width)?;
    overlay(&mut buffer, &ring, 0, 0);
    let cx = (buffer.width() / 2) as f32;
    let cy = (buffer.height() / 2) as f32;
    apply_transparency(&mut buffer, ring_side / 2, cx, cy);

    Ok(buffer)
}

/// Apply transparency to the image buffer pixels outside the ring
fn apply_transparency(buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, radius: u32, cx: f32, cy: f32) {
    buffer.enumerate_pixels_mut()
        .for_each(|(x, y, px)| {
            let distance = (x as f32 - cx).hypot(y as f32 - cy);
            if distance > radius as f32 {
                px[3] = 0;
            }
        });
}

fn get_ring_width(ring_img: &DynamicImage) -> u32 {
    // count non-transparent pixels along the top half the Y axis (in a single column)
    let x = ring_img.width() / 2;
    (0..ring_img.height() / 2)
        .map(|y| ring_img.get_pixel(x, y))
        .map(|pixel| if pixel[3] != 0 { 1u32 } else { 0u32 })// 1 for non-transparent pixel
        .sum()
}
