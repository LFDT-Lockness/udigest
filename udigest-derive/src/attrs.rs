use syn::ext::IdentExt;

pub mod kw {
    syn::custom_keyword!(root);
    syn::custom_keyword!(tag);
    syn::custom_keyword!(as_bytes);
    syn::custom_keyword!(bound);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(rename);
    syn::custom_keyword!(with);
}

pub enum Attr {
    Root(Root),
    Tag(Tag),
    AsBytes(AsBytes),
    Bound(Bound),
    Skip(Skip),
    Rename(Rename),
    With(With),
}

impl Attr {
    pub fn kw_span(&self) -> proc_macro2::Span {
        match self {
            Attr::Root(attr) => attr.root.span,
            Attr::Tag(attr) => attr.tag.span,
            Attr::AsBytes(attr) => attr.as_bytes.span,
            Attr::Bound(attr) => attr.bound.span,
            Attr::Skip(attr) => attr.skip.span,
            Attr::Rename(attr) => attr.rename.span,
            Attr::With(attr) => attr.with.span,
        }
    }
}

impl syn::parse::Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::root) {
            Root::parse(input).map(Attr::Root)
        } else if lookahead.peek(kw::tag) {
            Tag::parse(input).map(Attr::Tag)
        } else if lookahead.peek(kw::as_bytes) {
            AsBytes::parse(input).map(Attr::AsBytes)
        } else if lookahead.peek(kw::bound) {
            Bound::parse(input).map(Attr::Bound)
        } else if lookahead.peek(kw::skip) {
            Skip::parse(input).map(Attr::Skip)
        } else if lookahead.peek(kw::rename) {
            Rename::parse(input).map(Attr::Rename)
        } else if lookahead.peek(kw::with) {
            With::parse(input).map(Attr::With)
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct Root {
    pub root: kw::root,
    pub _eq: syn::Token![=],
    pub path: RootPath,
}

pub type RootPath = syn::punctuated::Punctuated<syn::Ident, syn::Token![::]>;

impl syn::parse::Parse for Root {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let root = input.parse::<kw::root>()?;
        let _eq = input.parse::<syn::Token![=]>()?;
        let path = RootPath::parse_separated_nonempty_with(input, syn::Ident::parse_any)?;

        Ok(Self { root, _eq, path })
    }
}

pub struct Tag {
    pub tag: kw::tag,
    pub _eq: syn::Token![=],
    pub value: syn::Expr,
}

impl syn::parse::Parse for Tag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag = input.parse()?;
        let _eq = input.parse()?;
        let value = input.parse()?;

        Ok(Self { tag, _eq, value })
    }
}

pub struct AsBytes {
    pub as_bytes: kw::as_bytes,
    pub _eq: Option<syn::Token![=]>,
    pub value: Option<syn::Expr>,
}

impl syn::parse::Parse for AsBytes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let as_bytes = input.parse()?;
        let mut _eq = None;
        let mut value = None;

        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Token![=]) {
            _eq = Some(input.parse()?);
            value = Some(input.parse()?);
        }

        Ok(Self {
            as_bytes,
            _eq,
            value,
        })
    }
}

pub struct Bound {
    pub bound: kw::bound,
    pub _eq: syn::Token![=],
    pub value: syn::LitStr,
}

impl syn::parse::Parse for Bound {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let bound = input.parse()?;
        let _eq = input.parse()?;
        let value = input.parse()?;

        Ok(Self { bound, _eq, value })
    }
}

pub struct Skip {
    pub skip: kw::skip,
}

impl syn::parse::Parse for Skip {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let skip = input.parse()?;
        Ok(Self { skip })
    }
}

pub struct Rename {
    pub rename: kw::rename,
    pub _eq: syn::Token![=],
    pub value: syn::Expr,
}

impl syn::parse::Parse for Rename {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let rename = input.parse()?;
        let _eq = input.parse()?;
        let value = input.parse()?;

        Ok(Self { rename, _eq, value })
    }
}

pub struct With {
    pub with: kw::with,
    pub _eq: syn::Token![=],
    pub value: syn::Expr,
}

impl syn::parse::Parse for With {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let with = input.parse()?;
        let _eq = input.parse()?;
        let value = input.parse()?;
        Ok(Self { with, _eq, value })
    }
}
