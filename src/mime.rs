use crate::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

/// A generic MIME Entity.
#[derive(Debug, PartialEq, Clone)]
pub struct RawEntity<'a> {
    pub mime_type: MimeType<'a>,
    /// The subtype (in lowercase).
    pub subtype: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    pub id: Option<(Cow<'a, str>, Cow<'a, str>)>,
    /// Parameters named in lowercase.
    pub parameters: HashMap<Cow<'a, str>, Cow<'a, str>>,
    #[cfg(feature = "content-disposition")]
    pub disposition: Option<Disposition<'a>>,
    /// The raw value of this entity.
    /// It has already been decoded.
    pub value: Cow<'a, [u8]>,
    pub additional_headers: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> RawEntity<'a> {
    /// Use this function to decode [text](Entity::Text) and [multipart](Entity::Multipart) values.\
    /// If this library is not able to provide a higher-level structure, the data will be returned [untouched]([Entity::Unknown]).\
    /// If this entity is supported but is wrongly formatted, an error will be returned.
    pub fn parse(self) -> Result<Entity<'a>, Error> {
        crate::parsing::mime::entity::entity(self)
    }
}

/// A higher-level reprentation of entities.\
/// Can be obtained with [RawEntity::parse].
#[derive(Debug, PartialEq, Clone)]
pub enum Entity<'a> {
    /// A multipart entity is an array of entities.\
    /// See the subtype for information about their relation.
    Multipart {
        subtype: Cow<'a, str>,
        content: Vec<RawEntity<'a>>,
    },
    /// A decoded text entity.\
    /// Supported charsets are all ISO, US-ASCII and UTF-8.
    Text {
        subtype: Cow<'a, str>,
        value: Cow<'a, str>,
    },
    /// All other entities that are not supported by this library.
    Unknown(Box<RawEntity<'a>>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum MimeType<'a> {
    // Fixme: rename to ContentType
    Text,
    Image,
    Audio,
    Video,
    Application,
    Message,
    Multipart,
    Other(Cow<'a, str>), // FIXME: rename to unknown
}

impl<'a> MimeType<'a> {
    pub fn into_owned(self) -> MimeType<'static> {
        match self {
            MimeType::Other(Cow::Owned(value)) => MimeType::Other(Cow::Owned(value)),
            MimeType::Other(Cow::Borrowed(value)) => MimeType::Other(Cow::Owned(value.to_owned())),
            MimeType::Text => MimeType::Text,
            MimeType::Image => MimeType::Image,
            MimeType::Audio => MimeType::Audio,
            MimeType::Video => MimeType::Video,
            MimeType::Application => MimeType::Application,
            MimeType::Message => MimeType::Message,
            MimeType::Multipart => MimeType::Multipart,
        }
    }
}

/// Information about how a [RawEntity] must be displayed.\
/// Is accessible from [Disposition::disposition_type].
#[derive(Debug, PartialEq, Clone)]
pub enum DispositionType<'a> {
    /// An inline entity\
    /// [Learn more](https://tools.ietf.org/html/rfc2183#section-2.1)
    Inline,
    /// An attachment\
    /// [Learn more](https://tools.ietf.org/html/rfc2183#section-2.2)
    Attachment,
    /// An unknown content-disposition. Should be treated as [DispositionType::Attachment].\
    /// [Learn more](https://tools.ietf.org/html/rfc2183#section-2.8).
    Unknown(Cow<'a, str>),
}

impl<'a> DispositionType<'a> {
    pub fn into_owned(self) -> DispositionType<'static> {
        match self {
            DispositionType::Unknown(Cow::Owned(value)) => {
                DispositionType::Unknown(Cow::Owned(value))
            }
            DispositionType::Unknown(Cow::Borrowed(value)) => {
                DispositionType::Unknown(Cow::Owned(value.to_owned()))
            }
            DispositionType::Inline => DispositionType::Inline,
            DispositionType::Attachment => DispositionType::Attachment,
        }
    }
}

/// Some information about how to display a [RawEntity] and some file metadata.\
/// Is accessible from [RawEntity::disposition].\
/// The size parameter is not directly supported as it is the "approximate size". You can get the exact size in bytes by calling `.len()` on the value of an [RawEntity::value].
#[derive(Debug, PartialEq, Clone)]
pub struct Disposition<'a> {
    pub disposition_type: DispositionType<'a>,
    pub filename: Option<Cow<'a, str>>,
    pub creation_date: Option<DateTime>,
    pub modification_date: Option<DateTime>,
    pub read_date: Option<DateTime>,
    pub unstructured: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a> Disposition<'a> {
    pub fn into_owned(self) -> Disposition<'static> {
        Disposition {
            disposition_type: self.disposition_type.into_owned(),
            filename: self
                .filename
                .map(|filename| Cow::Owned(filename.into_owned())),
            creation_date: self.creation_date,
            modification_date: self.modification_date,
            read_date: self.read_date,
            unstructured: self
                .unstructured
                .into_iter()
                .map(|(n, v)| (Cow::Owned(n.into_owned()), Cow::Owned(v.into_owned())))
                .collect(),
        }
    }
}

impl<'a> MimeType<'a> {
    pub fn is_composite_type(&self) -> bool {
        match self {
            MimeType::Message => true,
            MimeType::Multipart => true,
            MimeType::Text => false,
            MimeType::Image => false,
            MimeType::Audio => false,
            MimeType::Video => false,
            MimeType::Application => false,
            MimeType::Other(_) => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ContentTransferEncoding<'a> {
    SevenBit,
    HeightBit,
    Binary,
    QuotedPrintable,
    Base64,
    Other(Cow<'a, str>),
}

impl<'a> ContentTransferEncoding<'a> {
    pub fn into_owned(self) -> ContentTransferEncoding<'static> {
        match self {
            ContentTransferEncoding::Other(Cow::Owned(value)) => {
                ContentTransferEncoding::Other(Cow::Owned(value))
            }
            ContentTransferEncoding::Other(Cow::Borrowed(value)) => {
                ContentTransferEncoding::Other(Cow::Owned(value.to_owned()))
            }
            ContentTransferEncoding::SevenBit => ContentTransferEncoding::SevenBit,
            ContentTransferEncoding::HeightBit => ContentTransferEncoding::HeightBit,
            ContentTransferEncoding::Binary => ContentTransferEncoding::Binary,
            ContentTransferEncoding::QuotedPrintable => ContentTransferEncoding::QuotedPrintable,
            ContentTransferEncoding::Base64 => ContentTransferEncoding::Base64,
        }
    }
}
