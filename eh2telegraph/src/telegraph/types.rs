// Partly borrowed from https://github.com/Aloxaf/telegraph-rs/blob/master/src/types.rs

use serde::{Deserialize, Serialize};

/// This object represents a Telegraph account.
#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    /// Account name, helps users with several accounts remember which they are currently using.
    ///
    /// Displayed to the user above the "Edit/Publish" button on Telegra.ph, other users don't see this name.
    pub short_name: Option<String>,
    /// Default author name used when creating new articles.
    pub author_name: Option<String>,
    /// Profile link, opened when users click on the author's name below the title.
    ///
    /// Can be any link, not necessarily to a Telegram profile or channel.
    pub author_url: Option<String>,
    /// Optional. Only returned by the createAccount and revokeAccessToken method.
    ///
    /// Access token of the Telegraph account.
    pub access_token: Option<String>,
    /// Optional. URL to authorize a browser on telegra.ph and connect it to a Telegraph account.
    ///
    /// This URL is valid for only one use and for 5 minutes only.
    pub auth_url: Option<String>,
    /// Optional. Number of pages belonging to the Telegraph account.
    pub page_count: Option<i32>,
}

/// This object represents a list of Telegraph articles belonging to an account. Most recently created articles first.
#[derive(Debug, Clone, Deserialize)]
pub struct PageList {
    /// Total number of pages belonging to the target Telegraph account.
    pub total_count: i32,
    /// Requested pages of the target Telegraph account.
    pub pages: Vec<Page>,
}

/// This object represents a page to create on Telegraph.
#[derive(Debug, Clone, Serialize)]
pub struct PageCreate {
    /// Title of the page.
    pub title: String,
    /// Content of the page.
    pub content: Vec<Node>,

    /// Optional. Name of the author, displayed below the title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    /// Optional. Profile link, opened when users click on the author's name below the title.
    /// Can be any link, not necessarily to a Telegram profile or channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_url: Option<String>,
}

/// This object represents a page to edit on Telegraph.
#[derive(Debug, Clone, Serialize)]
pub struct PageEdit {
    /// Title of the page.
    pub title: String,
    /// Path to the page.
    pub path: String,
    /// Content of the page.
    pub content: Vec<Node>,

    /// Optional. Name of the author, displayed below the title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    /// Optional. Profile link, opened when users click on the author's name below the title.
    /// Can be any link, not necessarily to a Telegram profile or channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_url: Option<String>,
}

/// This object represents a page on Telegraph.
#[derive(Debug, Clone, Deserialize)]
pub struct Page {
    /// Path to the page.
    pub path: String,
    /// URL of the page.
    pub url: String,
    /// Title of the page.
    pub title: String,
    /// Description of the page.
    pub description: String,
    /// Optional. Name of the author, displayed below the title.
    pub author_name: Option<String>,
    /// Optional. Profile link, opened when users click on the author's name below the title.
    ///
    /// Can be any link, not necessarily to a Telegram profile or channel.
    pub author_url: Option<String>,
    /// Optional. Image URL of the page.
    pub image_url: Option<String>,
    /// Optional. Content of the page.
    pub content: Option<Vec<Node>>,
    /// Number of page views for the page.
    pub views: i32,
    /// Optional. Only returned if access_token passed.
    ///
    /// True, if the target Telegraph account can edit the page.
    pub can_edit: Option<bool>,
}

/// This object represents the number of page views for a Telegraph article.
#[derive(Debug, Clone, Deserialize)]
pub struct PageViews {
    /// Number of page views for the target page.
    pub views: i32,
}

/// This abstract object represents a DOM Node.
///
/// It can be a String which represents a DOM text node or a NodeElement object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Node {
    Text(String),
    NodeElement(NodeElement),
}

impl Node {
    // Estimate approximate size of serialized string.
    // We don't consider escape.
    pub fn estimate_size(&self) -> usize {
        match self {
            Node::Text(s) => s.len(),
            Node::NodeElement(e) => {
                // {"tag":"?","attrs":?,"children":?}
                // Init size for : {"tag":""} + tag length max(11)
                let mut size = 21;
                if let Some(attrs) = &e.attrs {
                    // size add: ,"attrs":{}
                    size += 11;
                    if let Some(href) = &attrs.href {
                        // size add: "href":""
                        size += 9 + href.len();
                    }
                    if let Some(src) = &attrs.src {
                        // size add: ,"src":""
                        size += 9 + src.len();
                    }
                }
                if let Some(children) = &e.children {
                    // size add: ,"children":[]
                    size += 14;
                    for child in children {
                        size += child.estimate_size() + 1;
                    }
                }
                size
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Tag {
    A,
    Aside,
    B,
    Blockquote,
    Br,
    Code,
    Em,
    Figcaption,
    Figure,
    H3,
    H4,
    Hr,
    I,
    Iframe,
    Img,
    Li,
    Ol,
    P,
    Pre,
    S,
    Strong,
    U,
    Ul,
    Video,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeElementAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
}

/// This object represents a DOM element node.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeElement {
    /// Name of the DOM element.
    /// Available tags: a, aside, b, blockquote, br, code, em, figcaption, figure, h3, h4, hr,
    /// i, iframe, img, li, ol, p, pre, s, strong, u, ul, video.
    pub tag: Tag,
    /// Optional. Attributes of the DOM element.
    ///
    /// Key of object represents name of attribute, value represents value of attribute.
    ///
    /// Available attributes: href, src.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<NodeElementAttr>,
    /// Optional. List of child nodes for the DOM element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Node>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MediaInfo {
    /// Path of the file uploaded.
    pub src: String,
}

impl From<Page> for PageEdit {
    fn from(p: Page) -> Self {
        Self {
            title: p.title,
            path: p.path,
            content: p.content.unwrap_or_default(),
            author_name: p.author_name,
            author_url: p.author_url,
        }
    }
}

impl Node {
    pub fn new_p_text<S: Into<String>>(text: S) -> Self {
        Node::NodeElement(NodeElement {
            tag: Tag::P,
            attrs: None,
            children: Some(vec![Node::Text(text.into())]),
        })
    }

    pub fn new_image<S: Into<String>>(src: S) -> Self {
        Node::NodeElement(NodeElement {
            tag: Tag::Img,
            attrs: Some(NodeElementAttr {
                src: Some(src.into()),
                href: None,
            }),
            children: None,
        })
    }
}

macro_rules! nt {
    ($s:expr) => {
        Node::Text($s.into())
    };
}

macro_rules! np {
    ($($n:expr),+) => {
        Node::NodeElement(NodeElement {
            tag: Tag::P,
            attrs: Some(NodeElementAttr {
                src: None,
                href: None,
            }),
            children: Some(vec![$($n),+]),
        })
    };
}

macro_rules! na {
    (@$href:expr,$($n:expr),+) => {
        Node::NodeElement(NodeElement {
            tag: Tag::A,
            attrs: Some(NodeElementAttr {
                src: None,
                href: Some($href.into()),
            }),
            children: Some(vec![$($n),+]),
        })
    };
}
