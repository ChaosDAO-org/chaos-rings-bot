use std::{env, fmt};
use std::borrow::Cow;
use std::io::Cursor;
use std::path::Path;

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
pub enum RingError {
    GenericError(String),
    UserRecoverableError(String),
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ring").description("Overlay a ChaosDAO ring to an avatar").create_option(
        |option| {
            option
                .name("avatar")
                .description("A square profile picture")
                .kind(CommandOptionType::Attachment)
                .required(true)
        },
    )
}

pub async fn run<'a>(user: &'a Member, user_image: &'a Attachment) -> Result<AttachmentType<'a>, RingError> {
    let user_role = find_dao_role(user)
        .ok_or_else(|| RingError::UserRecoverableError(String::from("No proper role found for user")))?;

    let ring_path = match user_role {
        DaoRole::Frens => { env::var("CHAOSRING_FRENS") }
        DaoRole::Regulars => { env::var("CHAOSRING_REGULARS") }
        DaoRole::DAOists => { env::var("CHAOSRING_DAOISTS") }
    }
        .map_err(|err| RingError::GenericError(err.to_string()))?;
    let ring_reader = ImageReader::open(Path::new(&ring_path))
        .map_err(|err| RingError::GenericError(err.to_string()))?;
    let ring = ring_reader.decode()
        .map_err(|err| RingError::GenericError(err.to_string()))?;

    let avatar = user_image.download()
        .await
        .map_err(|err| RingError::GenericError(err.to_string()))?;
    let avatar = image::load_from_memory(&avatar)
        .and_then(|avatar| overlay_ring(&avatar.to_rgba8(), &ring.to_rgba8()))
        .map_err(|err| RingError::GenericError(err.to_string()))?;

    let buf: Vec<u8> = Vec::with_capacity(avatar.as_raw().len());
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(buf);
    avatar.write_to(&mut cursor, ImageOutputFormat::Png)
        .map_err(|err| RingError::GenericError(err.to_string()))?;
    let attachment = AttachmentType::Bytes {
        // data: Cow::from(cursor.get_ref()),
        data: Cow::from(cursor.into_inner()),
        filename: String::from("avatar.png"),
    };

    Ok(attachment)
}

fn find_dao_role(member: &Member) -> Option<DaoRole> {
    let user_roles: &Vec<RoleId> = &member.roles;

    let fren: RoleId = RoleId(1023569411178770434); // Greenring
    let regular: RoleId = RoleId(1023569019422392401); // Redring
    let daoist: RoleId = RoleId(1023569488278458418); // Bluering

    if user_roles.contains(&daoist) {
        Some(DaoRole::DAOists)
    } else if user_roles.contains(&regular) {
        Some(DaoRole::Regulars)
    } else if user_roles.contains(&fren) {
        Some(DaoRole::Frens)
    } else {
        println!("No proper role found");
        None
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
    // buffer.save("test.png").unwrap();
    Ok(buffer)
}

fn get_ring_width(ring_img: &DynamicImage) -> u32 {
    // count non-transparent pixels along the top half the Y axis (in a single column)
    let x = ring_img.width() / 2;
    (0..ring_img.height() / 2)
        .map(|y| ring_img.get_pixel(x, y))
        .map(|pixel| if pixel[3] != 0 { 1u32 } else { 0u32 })// 1 for non-transparent pixel
        .sum()
}


impl fmt::Display for RingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RingError::GenericError(reason) => { write!(f, "Error while preparing an avatar: {}", reason) }
            RingError::UserRecoverableError(reason) => { write!(f, "Error while preparing an avatar: {}", reason) }
        }
    }
}
