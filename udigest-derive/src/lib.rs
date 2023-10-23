use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Error, Result};

mod attrs;

#[proc_macro_derive(Digestable, attributes(udigest))]
pub fn digestable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match digestable_inner(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn digestable_inner(input: syn::DeriveInput) -> Result<proc_macro2::TokenStream> {
    let mut container_attrs = ContainerAttrs::default();

    // Parse container-level attributes
    for attr in input.attrs {
        let Some(attr) = parse_attribute(&attr)? else {
            continue;
        };
        match attr {
            attrs::Attr::Root(_) if container_attrs.root.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"))
            }
            attrs::Attr::Root(attr) => {
                container_attrs.root = Some(attr);
            }
            attrs::Attr::Tag(_) if container_attrs.tag.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"))
            }
            attrs::Attr::Tag(attr) => {
                container_attrs.tag = Some(attr);
            }
            attrs::Attr::Bound(_) if container_attrs.bound.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
            }
            attrs::Attr::Bound(attr) => {
                container_attrs.bound = Some(attr);
            }
            _ => return Err(Error::new(attr.kw_span(), "attribute is not allowed here")),
        }
    }

    let s = match input.data {
        syn::Data::Struct(s) => s,
        syn::Data::Enum(e) => {
            return Err(Error::new(
                e.enum_token.span,
                "enums support is not implemented yet",
            ));
        }
        syn::Data::Union(u) => {
            return Err(Error::new(u.union_token.span, "unions are not supported"));
        }
    };

    let mut struct_fields = vec![];
    for (index, field) in (0..).zip(s.fields.iter()) {
        let mut field_attrs = FieldAttrs::default();

        let mem = field
            .ident
            .clone()
            .map(syn::Member::Named)
            .unwrap_or_else(|| {
                syn::Index {
                    index,
                    span: field.span(),
                }
                .into()
            });

        for attr in &field.attrs {
            let Some(attr) = parse_attribute(attr)? else {
                continue;
            };
            match attr {
                attrs::Attr::AsBytes(_) if field_attrs.as_bytes.is_some() => {
                    return Err(Error::new(attr.kw_span(), "attribute is duplicated"))
                }
                attrs::Attr::AsBytes(attr) => {
                    field_attrs.as_bytes = Some(attr);
                }
                attrs::Attr::Skip(_) if field_attrs.skip.is_some() => {
                    return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
                }
                attrs::Attr::Skip(attr) => {
                    field_attrs.skip = Some(attr);
                }
                _ => return Err(Error::new(attr.kw_span(), "attribute is not allowed here")),
            }
        }

        struct_fields.push(Field {
            attrs: field_attrs,
            mem,
        })
    }

    generate_impl_for_struct(
        &container_attrs,
        &input.ident,
        &input.generics,
        &struct_fields,
    )
}

fn generate_impl_for_struct(
    attrs: &ContainerAttrs,
    struct_name: &syn::Ident,
    struct_generics: &syn::Generics,
    struct_fields: &[Field],
) -> Result<proc_macro2::TokenStream> {
    let root_path = attrs.get_root_path();
    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();

    let where_clause = match &attrs.bound {
        Some(bound) => {
            let overriden_where_clause: proc_macro2::TokenStream = bound
                .value
                .value()
                .parse()
                .map_err(|err| Error::new(bound.value.span(), err))?;
            quote_spanned! {bound.value.span() =>
                where #overriden_where_clause
            }
        }
        None => {
            let predicates = where_clause.map(|w| &w.predicates);

            let generated_predicates = struct_generics.type_params().map(|g| {
                let ident = &g.ident;
                quote! {#ident: #root_path::Digestable,}
            });

            quote! {
                where #(#generated_predicates)* #predicates
            }
        }
    };

    let specify_tag = attrs.tag.as_ref().map(|attrs::Tag { value, .. }| {
        quote_spanned! {value.span() =>
            let tag = #value;
            let tag = AsRef::<[u8]>::as_ref(&tag);
            encoder.set_tag(tag);
        }
    });

    let encode_each_field = struct_fields.iter().map(|f| {
        if f.attrs.skip.is_some() {
            return quote! {};
        }

        let field_name = f.stringify_field_name();
        let mem = &f.mem;

        match &f.attrs.as_bytes {
            Some(attr) => match &attr.value {
                Some(func) => quote_spanned! {f.mem.span() => {
                    let field_encoder = encoder.add_field(#field_name);
                    let field_bytes = #func(&self.#mem);
                    let field_bytes = AsRef::<[u8]>::as_ref(field_bytes);
                    field_encoder.encode_leaf().chain(field_bytes);
                }},
                None => quote_spanned!(f.mem.span() => {
                    let field_encoder = encoder.add_field(#field_name);
                    let field_bytes: &[u8] = AsRef::<[u8]>::as_ref(&self.#mem);
                    field_encoder.encode_leaf().chain(field_bytes);
                }),
            },
            None => quote_spanned! {f.mem.span() => {
                let field_encoder = encoder.add_field(#field_name);
                #root_path::Digestable::unambiguously_encode(&self.#mem, field_encoder);
            }},
        }
    });

    Ok(quote! {
        impl #impl_generics #root_path::Digestable for #struct_name #ty_generics #where_clause {
            fn unambiguously_encode<B>(&self, encoder: #root_path::encoding::EncodeValue<B>)
            where
                B: #root_path::Buffer
            {
                let mut encoder = encoder.encode_struct();
                #specify_tag
                #(#encode_each_field)*
                encoder.finish();
            }
        }
    })
}

fn parse_attribute(attr: &syn::Attribute) -> Result<Option<attrs::Attr>> {
    let attr_tokens = match &attr.meta {
        syn::Meta::List(meta) if meta.path.is_ident("udigest") => &meta.tokens,
        syn::Meta::Path(path) if path.is_ident("udigest") => {
            return Err(Error::new(
                path.span(),
                "empty attribute doesn't make sense",
            ))
        }
        syn::Meta::NameValue(meta) if meta.path.is_ident("udigest") => {
            return Err(Error::new(
                meta.value.span(),
                "attribute needs to be specified in parentheses (e.g. `#[udigest(skip)]`)",
            ));
        }
        _ => return Ok(None),
    };
    syn::parse2(attr_tokens.clone()).map(Some)
}

#[derive(Default)]
struct ContainerAttrs {
    root: Option<attrs::Root>,
    tag: Option<attrs::Tag>,
    bound: Option<attrs::Bound>,
}

impl ContainerAttrs {
    pub fn get_root_path(&self) -> attrs::RootPath {
        self.root
            .as_ref()
            .map(|root| root.path.clone())
            .unwrap_or_else(|| {
                [syn::Ident::new("udigest", proc_macro2::Span::call_site())]
                    .into_iter()
                    .collect()
            })
    }
}

#[derive(Default)]
struct FieldAttrs {
    as_bytes: Option<attrs::AsBytes>,
    skip: Option<attrs::Skip>,
}

struct Field {
    attrs: FieldAttrs,
    mem: syn::Member,
}

impl Field {
    pub fn stringify_field_name(&self) -> String {
        match &self.mem {
            syn::Member::Named(ident) => ident.to_string(),
            syn::Member::Unnamed(index) => index.index.to_string(),
        }
    }
}
