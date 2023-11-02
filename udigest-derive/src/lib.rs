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

    match input.data {
        syn::Data::Struct(s) => process_struct(&container_attrs, &input.ident, &input.generics, &s),
        syn::Data::Enum(e) => process_enum(&container_attrs, &input.ident, &input.generics, &e),
        syn::Data::Union(u) => Err(Error::new(u.union_token.span, "unions are not supported")),
    }
}

fn process_enum(
    attrs: &ContainerAttrs,
    name: &syn::Ident,
    generics: &syn::Generics,
    e: &syn::DataEnum,
) -> Result<proc_macro2::TokenStream> {
    let variants = e
        .variants
        .iter()
        .map(|v| {
            Ok(Variant {
                name: v.ident.clone(),
                ty: match &v.fields {
                    syn::Fields::Named(_) => VariantType::Named,
                    syn::Fields::Unnamed(_) => VariantType::Unnamed,
                    syn::Fields::Unit => VariantType::Unit,
                },
                fields: (0..)
                    .zip(v.fields.iter())
                    .map(|(i, f)| process_field(i, f))
                    .collect::<Result<Vec<_>>>()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    generate_impl_for_enum(attrs, name, generics, &variants)
}

fn process_struct(
    container_attrs: &ContainerAttrs,
    name: &syn::Ident,
    generics: &syn::Generics,
    s: &syn::DataStruct,
) -> Result<proc_macro2::TokenStream> {
    let struct_fields = (0..)
        .zip(s.fields.iter())
        .map(|(i, f)| process_field(i, f))
        .collect::<Result<Vec<_>>>()?;

    generate_impl_for_struct(container_attrs, name, generics, &struct_fields)
}

fn process_field(index: u32, field: &syn::Field) -> Result<Field> {
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
            attrs::Attr::Rename(_) if field_attrs.rename.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"))
            }
            attrs::Attr::Rename(attr) => {
                field_attrs.rename = Some(attr);
            }
            _ => return Err(Error::new(attr.kw_span(), "attribute is not allowed here")),
        }
    }

    Ok(Field {
        attrs: field_attrs,
        mem,
    })
}

fn generate_impl_for_enum(
    attrs: &ContainerAttrs,
    enum_name: &syn::Ident,
    enum_generics: &syn::Generics,
    enum_variants: &[Variant],
) -> Result<proc_macro2::TokenStream> {
    let root_path = attrs.get_root_path();
    let (impl_generics, ty_generics, _) = enum_generics.split_for_impl();

    let where_clause = make_where_clause(attrs, enum_generics)?;

    let encoder_var = syn::Ident::new("encoder", proc_macro2::Span::call_site());

    let specify_tag = attrs.tag.as_ref().map(|attrs::Tag { value, .. }| {
        quote_spanned! {value.span() =>
            let tag = #value;
            let tag = AsRef::<[u8]>::as_ref(&tag);
            #encoder_var.set_tag(tag);
        }
    });

    let match_expr = if !enum_variants.is_empty() {
        let match_branches = enum_variants.iter().map(|v| {
            let variant_name = &v.name;
            let field_bindings = (0..v.fields.len())
                .map(|i| syn::Ident::new(&format!("field{i}"), proc_macro2::Span::call_site()))
                .collect::<Vec<_>>();
            let pattern = match v.ty {
                VariantType::Named => {
                    let fields = v.fields.iter().zip(&field_bindings).map(|(f, binding)| {
                        let field_name = &f.mem;
                        quote! { #field_name: #binding }
                    });
                    quote! { {#(#fields),*} }
                }
                VariantType::Unnamed => {
                    let fields = field_bindings.iter().map(|binding| {
                        quote! {#binding}
                    });
                    quote! { (#(#fields),*) }
                }
                VariantType::Unit => {
                    quote!()
                }
            };

            let encode_fields = field_bindings.iter().zip(&v.fields).map(|(binding, f)| {
                encode_field(
                    &root_path,
                    &encoder_var,
                    &f.attrs,
                    f.mem.span(),
                    &f.stringify_field_name(),
                    &binding,
                )
            });

            let variant_name_str = variant_name.to_string();
            quote_spanned! {variant_name.span() =>
                #enum_name::#variant_name #pattern => {
                    let mut #encoder_var = #encoder_var.with_variant(#variant_name_str);
                    #(#encode_fields)*
                }
            }
        });
        quote! {
            match self {
                #(#match_branches)*
            }
        }
    } else {
        quote! {
            match *self {}
        }
    };

    Ok(quote! {
        impl #impl_generics #root_path::Digestable for #enum_name #ty_generics #where_clause {
            fn unambiguously_encode<B>(&self, encoder: #root_path::encoding::EncodeValue<B>)
            where
                B: #root_path::Buffer
            {
                let mut #encoder_var = encoder.encode_enum();
                #specify_tag
                #match_expr
            }
        }
    })
}

fn generate_impl_for_struct(
    attrs: &ContainerAttrs,
    struct_name: &syn::Ident,
    struct_generics: &syn::Generics,
    struct_fields: &[Field],
) -> Result<proc_macro2::TokenStream> {
    let root_path = attrs.get_root_path();
    let (impl_generics, ty_generics, _) = struct_generics.split_for_impl();

    let where_clause = make_where_clause(attrs, struct_generics)?;

    let specify_tag = attrs.tag.as_ref().map(|attrs::Tag { value, .. }| {
        quote_spanned! {value.span() =>
            let tag = #value;
            let tag = AsRef::<[u8]>::as_ref(&tag);
            encoder.set_tag(tag);
        }
    });

    let encoder_var = syn::Ident::new("encoder", proc_macro2::Span::call_site());
    let encode_each_field = struct_fields.iter().map(|f| {
        let mem = &f.mem;
        encode_field(
            &root_path,
            &encoder_var,
            &f.attrs,
            f.mem.span(),
            &f.stringify_field_name(),
            &quote_spanned! {f.mem.span() => &self.#mem},
        )
    });

    Ok(quote! {
        impl #impl_generics #root_path::Digestable for #struct_name #ty_generics #where_clause {
            fn unambiguously_encode<B>(&self, encoder: #root_path::encoding::EncodeValue<B>)
            where
                B: #root_path::Buffer
            {
                let mut #encoder_var = encoder.encode_struct();
                #specify_tag
                #(#encode_each_field)*
                #encoder_var.finish();
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

/// Takes the generics defined for the data type, produces a where clause that should
/// be used for trait implementation
///
/// If `bound` attribute is not specified, it takes where clause defined for datatype,
/// and populates it with constraints `A: Digestable` for every generic type defined for
/// the structure
///
/// If `bound` attribute is specified, it fully overrides the where clause
fn make_where_clause(
    attrs: &ContainerAttrs,
    generics: &syn::Generics,
) -> Result<proc_macro2::TokenStream> {
    let root_path = attrs.get_root_path();
    let predicates = generics.where_clause.as_ref().map(|w| &w.predicates);

    let generated_predicates = match &attrs.bound {
        Some(bound) => {
            let overriden_where_clause: proc_macro2::TokenStream = bound
                .value
                .value()
                .parse()
                .map_err(|err| Error::new(bound.value.span(), err))?;
            let predicates = syn::parse::Parser::parse2(
                syn::punctuated::Punctuated::<syn::WherePredicate, syn::Token![,]>::parse_terminated, 
                overriden_where_clause
            )
            .map_err(|err| Error::new(bound.value.span(), err))?;
            let predicates = predicates.iter();
            quote_spanned! {bound.value.span() =>
                #(#predicates,)*
            }
        }
        None => {
            let generated_predicates = generics.type_params().map(|g| {
                let ident = &g.ident;
                quote! {#ident: #root_path::Digestable,}
            });
            quote! { #(#generated_predicates)* }
        }
    };
    Ok(quote! {
        where #generated_predicates #predicates
    })
}

/// Generates a code that encodes a field into `encoder_var`
///
/// `field_name` represents a stringified name of the field, `field_ref` contains
/// expression that yields a reference to the field. `field_span` specifies a span
/// of the field, and `field_attrs` specifies field-level attributes.
///
/// `root_path` specifies a path to the `udigest` crate.
fn encode_field(
    root_path: &attrs::RootPath,
    encoder_var: &syn::Ident,
    field_attrs: &FieldAttrs,
    field_span: proc_macro2::Span,
    field_name: &str,
    field_ref: &impl quote::ToTokens,
) -> proc_macro2::TokenStream {
    if field_attrs.skip.is_some() {
        return quote! {};
    }

    let field_name = match &field_attrs.rename {
        None => quote! { #field_name },
        Some(attrs::Rename { rename, value, .. }) => quote_spanned! { rename.span => #value },
    };

    match &field_attrs.as_bytes {
        Some(attr) => match &attr.value {
            Some(func) => quote_spanned! {field_span => {
                let field_encoder = #encoder_var.add_field(#field_name);
                let field_bytes = #func(#field_ref);
                let field_bytes = AsRef::<[u8]>::as_ref(field_bytes);
                field_encoder.encode_leaf().chain(field_bytes);
            }},
            None => quote_spanned!(field_span => {
                let field_encoder = #encoder_var.add_field(#field_name);
                let field_bytes: &[u8] = AsRef::<[u8]>::as_ref(#field_ref);
                field_encoder.encode_leaf().chain(field_bytes);
            }),
        },
        None => quote_spanned! {field_span => {
            let field_encoder = #encoder_var.add_field(#field_name);
            #root_path::Digestable::unambiguously_encode(#field_ref, field_encoder);
        }},
    }
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
    rename: Option<attrs::Rename>,
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

struct Variant {
    name: syn::Ident,
    fields: Vec<Field>,
    ty: VariantType,
}

#[derive(PartialEq, Eq)]
enum VariantType {
    Named,
    Unnamed,
    Unit,
}
