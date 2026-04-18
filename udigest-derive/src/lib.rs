//! ## Proc macro for `udigest` crate
//!
//! This crate contains a proc macro for implementing `Digestable` trait
//! from [udigest crate](https://docs.rs/udigest), please refer to its
//! documentation.

use quote::{quote, quote_spanned};
use syn::{Error, Result, spanned::Spanned};

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
    let inferred_paths = infer_paths(&input);
    let mut container_attrs = ContainerAttrs::new(inferred_paths);

    // Parse container-level attributes
    for attr in input.attrs {
        let Some(attr) = parse_attribute(&attr)? else {
            continue;
        };
        match attr {
            attrs::Attr::Root(_) if container_attrs.root.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
            }
            attrs::Attr::Root(attr) => {
                container_attrs.root = Some(attr);
            }
            attrs::Attr::Tag(_) if container_attrs.tag.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
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

fn infer_paths(input: &syn::DeriveInput) -> InferredPaths {
    if let Some(derive_trait_path) = infer_trait_path_from_derive(input) {
        let inferred_root_path = derive_trait_path_to_root_path(&derive_trait_path)
            .unwrap_or_else(infer_root_path_from_dependencies);
        return InferredPaths {
            inferred_root_path,
            inferred_trait_path: derive_trait_path,
        };
    }

    let inferred_root_path = infer_root_path_from_dependencies();
    let inferred_trait_path = root_path_to_trait_path(&inferred_root_path);
    InferredPaths {
        inferred_root_path,
        inferred_trait_path,
    }
}

fn infer_root_path_from_dependencies() -> attrs::RootPath {
    let candidates = ["udigest", "udigest-encoding"]
        .into_iter()
        .filter_map(infer_root_from_dependency)
        .collect::<Vec<_>>();

    if let Some((path, _)) = candidates.iter().find(|(_, is_itself)| *is_itself) {
        return path.clone();
    }

    candidates
        .first()
        .map(|(path, _)| path.clone())
        .unwrap_or_else(default_root_path)
}

fn infer_trait_path_from_derive(input: &syn::DeriveInput) -> Option<attrs::RootPath> {
    for attr in &input.attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }

        let derive_paths = attr
            .parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .ok()?;

        for path in derive_paths {
            if path
                .segments
                .last()
                .map(|segment| segment.ident == "Digestable")
                .unwrap_or(false)
            {
                return Some(path);
            }
        }
    }

    None
}

fn derive_trait_path_to_root_path(path: &attrs::RootPath) -> Option<attrs::RootPath> {
    if path.segments.len() < 2 {
        return None;
    }

    let mut root = path.clone();
    root.segments.pop();
    Some(root)
}

fn root_path_to_trait_path(root: &attrs::RootPath) -> attrs::RootPath {
    let mut trait_path = root.clone();
    trait_path.segments.push(syn::PathSegment {
        ident: syn::Ident::new("Digestable", root.span()),
        arguments: syn::PathArguments::None,
    });
    trait_path
}

fn infer_root_from_dependency(dep: &'static str) -> Option<(attrs::RootPath, bool)> {
    let dep_to_ident = |name: &str| name.replace('-', "_");

    match proc_macro_crate::crate_name(dep).ok()? {
        proc_macro_crate::FoundCrate::Itself => Some((
            syn::parse_str::<syn::Path>(&format!("::{}", dep_to_ident(dep))).ok()?,
            true,
        )),
        proc_macro_crate::FoundCrate::Name(name) => Some((
            syn::parse_str::<syn::Path>(&format!("::{name}")).ok()?,
            false,
        )),
    }
}

fn default_root_path() -> attrs::RootPath {
    syn::parse_quote!(::udigest)
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
            let mut variant_attrs = EnumVariantAttrs::default();

            for attr in &v.attrs {
                let Some(attr) = parse_attribute(attr)? else {
                    continue;
                };
                match attr {
                    attrs::Attr::Rename(_) if variant_attrs.rename.is_some() => {
                        return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
                    }
                    attrs::Attr::Rename(attr) => {
                        variant_attrs.rename = Some(attr);
                    }
                    attrs::Attr::Root(_)
                    | attrs::Attr::Tag(_)
                    | attrs::Attr::AsBytes(_)
                    | attrs::Attr::Bound(_)
                    | attrs::Attr::Skip(_)
                    | attrs::Attr::With(_)
                    | attrs::Attr::As(_) => {
                        return Err(Error::new(
                            attr.kw_span(),
                            "not supported as attribute for enum variant",
                        ));
                    }
                }
            }
            Ok(Variant {
                attrs: variant_attrs,
                name: v.ident.clone(),
                ty: match &v.fields {
                    syn::Fields::Named(_) => VariantType::Named,
                    syn::Fields::Unnamed(_) => VariantType::Unnamed,
                    syn::Fields::Unit => VariantType::Unit,
                },
                fields: (0..)
                    .zip(v.fields.iter())
                    .map(|(i, f)| process_field(&attrs.get_root_path(), i, f))
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
        .map(|(i, f)| process_field(&container_attrs.get_root_path(), i, f))
        .collect::<Result<Vec<_>>>()?;

    generate_impl_for_struct(container_attrs, name, generics, &struct_fields)
}

fn process_field(root_path: &attrs::RootPath, index: u32, field: &syn::Field) -> Result<Field> {
    // same_ty = <root_path>::as_::Same
    let same_ty = {
        let mut root = root_path.clone();
        root.segments.push(syn::PathSegment {
            ident: syn::Ident::new("as_", root_path.span()),
            arguments: syn::PathArguments::None,
        });
        root.segments.push(syn::PathSegment {
            ident: syn::Ident::new("Same", root_path.span()),
            arguments: syn::PathArguments::None,
        });
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: root,
        })
    };
    let mut field_attrs = FieldAttrs::default();

    // FIXME: ideally, we want to use `field.span()`, however, that works awfully because
    // `proc_macro::Span::join` is not stabilized. E.g. if field is `name: Vec<Something>`,
    // `field.span()` will point to `name: Vec` instead of all field.
    //
    // We should change this once `Span::join` is stable.
    let field_span = field
        .ident
        .as_ref()
        .map(|i| i.span())
        .unwrap_or_else(|| field.span());

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
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
            }
            attrs::Attr::With(_) if field_attrs.with.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
            }
            attrs::Attr::Skip(_) if field_attrs.skip.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
            }
            attrs::Attr::Rename(_) if field_attrs.rename.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
            }
            attrs::Attr::As(_) if field_attrs.as_.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
            }
            attrs::Attr::AsBytes(_)
            | attrs::Attr::With(_)
            | attrs::Attr::As(_)
            | attrs::Attr::Skip(_)
                if count_trues([
                    field_attrs.as_bytes.is_some(),
                    field_attrs.with.is_some(),
                    field_attrs.as_.is_some(),
                    field_attrs.skip.is_some(),
                ]) > 0 =>
            {
                return Err(Error::new(
                    attr.kw_span(),
                    "attributes `with`, `as_bytes`, `as` and 'skip` cannot be used together",
                ));
            }
            attrs::Attr::AsBytes(attr) => {
                field_attrs.as_bytes = Some(attr);
            }
            attrs::Attr::With(attr) => {
                field_attrs.with = Some(attr);
            }
            attrs::Attr::Skip(attr) => {
                field_attrs.skip = Some(attr);
            }
            attrs::Attr::Rename(attr) => {
                field_attrs.rename = Some(attr);
            }
            attrs::Attr::As(mut attr) => {
                attr.value = type_replace_infer(attr.value, same_ty.clone())?;
                field_attrs.as_ = Some(attr);
            }
            _ => return Err(Error::new(attr.kw_span(), "attribute is not allowed here")),
        }
    }

    Ok(Field {
        span: field_span,
        attrs: field_attrs,
        mem,
        ty: field.ty.clone(),
    })
}

fn count_trues(i: impl IntoIterator<Item = bool>) -> usize {
    i.into_iter().filter(|x| *x).count()
}

/// Traverses the type and replaces `_` with `infer_ty`
///
/// E.g. `Option<_>` becomes `Option<{infer_ty}>`.
///
/// Returns an error if provided type is not supported. It supports any types that
/// can be found as the type of field in the struct. For instance, `impl Trait` is
/// not supported.
///
/// The function only traverses some types such as: path type (e.g. `std::result::Result<T, E>`),
/// arrays, slices, tuples, references, pointers. It does not traverse anything else,
/// like function pointers or trait objects. E.g. `fn(_) -> u32` or `Box<dyn _>` are
/// not modified by the function.
fn type_replace_infer(ty: syn::Type, infer_ty: syn::Type) -> Result<syn::Type> {
    match ty {
        syn::Type::Infer(_) => Ok(infer_ty),

        syn::Type::Array(ty) => Ok(syn::Type::Array(syn::TypeArray {
            bracket_token: ty.bracket_token,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
            semi_token: ty.semi_token,
            len: ty.len,
        })),
        syn::Type::Group(ty) => Ok(syn::Type::Group(syn::TypeGroup {
            group_token: ty.group_token,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
        })),
        syn::Type::Paren(ty) => Ok(syn::Type::Paren(syn::TypeParen {
            paren_token: ty.paren_token,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
        })),
        syn::Type::Path(ty) => Ok(syn::Type::Path(syn::TypePath {
            qself: ty.qself,
            path: syn::Path {
                leading_colon: ty.path.leading_colon,
                // Traverse each segment of the path, e.g.:
                //
                // std::result::Result<T, E>
                // 1 --| 2 ----| 3 --------|
                segments: ty
                    .path
                    .segments
                    .into_pairs()
                    .map(|pair| {
                        let (seg, sep) = pair.into_tuple();
                        let args = match seg.arguments {
                            syn::PathArguments::None => syn::PathArguments::None,
                            syn::PathArguments::Parenthesized(x) => {
                                return Err(Error::new(x.span(), "not allowed in this context"));
                            }
                            // Result<T, E>
                            //       ^----^ angle-bracketed arguments
                            syn::PathArguments::AngleBracketed(args) => {
                                syn::PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments {
                                        colon2_token: args.colon2_token,
                                        lt_token: args.lt_token,
                                        // traverse each path argument
                                        args: args
                                            .args
                                            .into_pairs()
                                            .map(|pair| {
                                                let (arg, comma) = pair.into_tuple();
                                                let arg = match arg {
                                                    // type argument => need to traverse
                                                    syn::GenericArgument::Type(ty) => {
                                                        syn::GenericArgument::Type(
                                                            type_replace_infer(
                                                                ty,
                                                                infer_ty.clone(),
                                                            )?,
                                                        )
                                                    }
                                                    // other arguments we do not care about, like lifetimes
                                                    _ => arg,
                                                };
                                                Ok(syn::punctuated::Pair::new(arg, comma))
                                            })
                                            .collect::<Result<_>>()?,
                                        gt_token: args.gt_token,
                                    },
                                )
                            }
                        };

                        Ok(syn::punctuated::Pair::new(
                            syn::PathSegment {
                                ident: seg.ident,
                                arguments: args,
                            },
                            sep,
                        ))
                    })
                    .collect::<Result<_>>()?,
            },
        })),
        syn::Type::Ptr(ty) => Ok(syn::Type::Ptr(syn::TypePtr {
            star_token: ty.star_token,
            const_token: ty.const_token,
            mutability: ty.mutability,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
        })),
        syn::Type::Reference(ty) => Ok(syn::Type::Reference(syn::TypeReference {
            and_token: ty.and_token,
            lifetime: ty.lifetime,
            mutability: ty.mutability,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
        })),
        syn::Type::Slice(ty) => Ok(syn::Type::Slice(syn::TypeSlice {
            bracket_token: ty.bracket_token,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
        })),
        syn::Type::Tuple(ty) => Ok(syn::Type::Tuple(syn::TypeTuple {
            paren_token: ty.paren_token,
            // Traverse each type in the tuple
            elems: ty
                .elems
                .into_pairs()
                .map(|pair| {
                    let (ty, comma) = pair.into_tuple();
                    let ty = type_replace_infer(ty, infer_ty.clone())?;
                    Ok(syn::punctuated::Pair::new(ty, comma))
                })
                .collect::<Result<_>>()?,
        })),

        // Following types are not traversed
        syn::Type::BareFn(_)
        | syn::Type::Macro(_)
        | syn::Type::Never(_)
        | syn::Type::TraitObject(_)
        | syn::Type::Verbatim(_) => Ok(ty),

        // Following types are not supported
        syn::Type::ImplTrait(_) => Err(Error::new(
            ty.span(),
            "`impl Trait` is not supported in this context",
        )),

        // This might happen if Rust gets a new type in the future
        _ => Err(Error::new(ty.span(), "unknown type")),
    }
}

fn generate_impl_for_enum(
    attrs: &ContainerAttrs,
    enum_name: &syn::Ident,
    enum_generics: &syn::Generics,
    enum_variants: &[Variant],
) -> Result<proc_macro2::TokenStream> {
    let root_path = attrs.get_root_path();
    let trait_path = attrs.get_trait_path();
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

    let variant_assertions =
        assert_no_duplicates_in_enum_variant_names(&attrs.get_root_path(), enum_variants);
    let (match_expr, fields_assertions) = if !enum_variants.is_empty() {
        let (match_branches, assertions) = enum_variants
            .iter()
            .map(|v| {
                let variant_name = &v.name;
                let span = variant_name.span();
                let field_bindings = v
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let prefix = if f.attrs.skip.is_some() { "_" } else { "" };
                        syn::Ident::new(&format!("{prefix}field{i}"), span)
                    })
                    .collect::<Vec<_>>();
                let pattern = match v.ty {
                    VariantType::Named => {
                        let fields = v.fields.iter().zip(&field_bindings).map(|(f, binding)| {
                            let field_name = &f.mem;
                            quote_spanned! { span => #field_name: #binding }
                        });
                        quote_spanned! { span => {#(#fields),*} }
                    }
                    VariantType::Unnamed => {
                        let fields = field_bindings.iter().map(|binding| {
                            quote_spanned! { span => #binding }
                        });
                        quote_spanned! { span => (#(#fields),*) }
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
                        f.span,
                        &f.stringify_field_name(),
                        &f.ty,
                        &binding,
                    )
                });

                let variant_name_encoding = if let Some(attr) = &v.attrs.rename {
                    quote::ToTokens::to_token_stream(&attr.value)
                } else {
                    let name = variant_name.to_string();
                    quote_spanned!(span => #name)
                };

                let field_names_duplicates_detection =
                    assert_no_duplicates_in_enum_variant_field_names(
                        &attrs.get_root_path(),
                        &variant_name.to_string(),
                        &v.fields,
                    );

                let match_branch = quote_spanned! {span =>
                    #enum_name::#variant_name #pattern => {
                        let mut #encoder_var = #encoder_var.with_variant(#variant_name_encoding);
                        #(#encode_fields)*
                    }
                };

                Ok((match_branch, field_names_duplicates_detection))
            })
            .collect::<Result<(Vec<_>, Vec<_>)>>()?;

        let match_expr = quote! {
            match self {
                #(#match_branches)*
            }
        };
        (match_expr, assertions)
    } else {
        (
            quote! {
                match *self {}
            },
            vec![],
        )
    };

    Ok(quote! {
        impl #impl_generics #trait_path for #enum_name #ty_generics #where_clause {
            fn unambiguously_encode(&self, encoder: #root_path::encoding::EncodeValue)
            {
                let mut #encoder_var = encoder.encode_enum();
                #specify_tag
                #match_expr
            }
        }
        const _: () = {
            #variant_assertions
            #(#fields_assertions)*
        };
    })
}

fn generate_impl_for_struct(
    attrs: &ContainerAttrs,
    struct_name: &syn::Ident,
    struct_generics: &syn::Generics,
    struct_fields: &[Field],
) -> Result<proc_macro2::TokenStream> {
    let root_path = attrs.get_root_path();
    let trait_path = attrs.get_trait_path();
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
            f.span,
            &f.stringify_field_name(),
            &f.ty,
            &quote_spanned! {f.span => &self.#mem},
        )
    });

    let field_names_duplications_detection =
        assert_no_duplicates_in_struct_field_names(&attrs.get_root_path(), struct_fields);

    Ok(quote! {
        impl #impl_generics #trait_path for #struct_name #ty_generics #where_clause {
            fn unambiguously_encode(&self, encoder: #root_path::encoding::EncodeValue)
            {
                let mut #encoder_var = encoder.encode_struct();
                #specify_tag
                #(#encode_each_field)*
                #encoder_var.finish();
            }
        }
        const _: () = {
            #field_names_duplications_detection
        };
    })
}

fn parse_attribute(attr: &syn::Attribute) -> Result<Option<attrs::Attr>> {
    let attr_tokens = match &attr.meta {
        syn::Meta::List(meta) if meta.path.is_ident("udigest") => &meta.tokens,
        syn::Meta::Path(path) if path.is_ident("udigest") => {
            return Err(Error::new(
                path.span(),
                "empty attribute doesn't make sense",
            ));
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
    let trait_path = attrs.get_trait_path();
    let predicates = generics.where_clause.as_ref().map(|w| &w.predicates);

    let generated_predicates = match &attrs.bound {
        Some(bound) => {
            let overridden_where_clause: proc_macro2::TokenStream = bound
                .value
                .value()
                .parse()
                .map_err(|err| Error::new(bound.value.span(), err))?;
            let predicates = syn::parse::Parser::parse2(
                syn::punctuated::Punctuated::<syn::WherePredicate, syn::Token![,]>::parse_terminated,
                overridden_where_clause
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
                quote! {#ident: #trait_path,}
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
    field_type: &syn::Type,
    field_ref: &impl quote::ToTokens,
) -> proc_macro2::TokenStream {
    if field_attrs.skip.is_some() {
        return quote! {};
    }

    let field_name = match &field_attrs.rename {
        None => quote! { #field_name },
        Some(attrs::Rename { rename, value, .. }) => quote_spanned! { rename.span => #value },
    };

    match (&field_attrs.as_bytes, &field_attrs.with, &field_attrs.as_) {
        (Some(attr), None, None) => match &attr.value {
            Some(func) => quote_spanned! {field_span => {
                let field_encoder = #encoder_var.add_field(#field_name);
                let field_bytes = #func(#field_ref);
                let field_bytes = AsRef::<[u8]>::as_ref(&field_bytes);
                field_encoder.encode_leaf_value(field_bytes);
            }},
            None => quote_spanned!(field_span => {
                let field_encoder = #encoder_var.add_field(#field_name);
                let field_bytes: &[u8] = AsRef::<[u8]>::as_ref(#field_ref);
                field_encoder.encode_leaf_value(field_bytes);
            }),
        },
        (None, Some(attrs::With { value: func, .. }), None) => quote_spanned! {field_span => {
            let field_encoder = #encoder_var.add_field(#field_name);
            #[allow(clippy::needless_borrow, clippy::needless_borrows_for_generic_args)]
            #func(#field_ref, field_encoder);
        }},
        (None, None, Some(attrs::As { value: ty, .. })) => quote_spanned! {field_span => {
            let field_encoder = #encoder_var.add_field(#field_name);
            #[allow(clippy::needless_borrow, clippy::needless_borrows_for_generic_args)]
            <#ty as #root_path::DigestAs<#field_type>>::digest_as(#field_ref, field_encoder)
        }},
        (None, None, None) => quote_spanned! {field_span => {
            let field_encoder = #encoder_var.add_field(#field_name);
            #root_path::Digestable::unambiguously_encode(#field_ref, field_encoder);
        }},
        _ => {
            unreachable!(
                "it should have been validated that `with`, `as_bytes`, `as` are not used in the same time"
            )
        }
    }
}

/// Produces a code which statically asserts that there's no duplicates in field name encodings
///
/// Takes a `root_path` (path to `udigest` lib), and list of fields
fn assert_no_duplicates_in_struct_field_names(
    root_path: &attrs::RootPath,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    assert_no_duplicates_in_field_names(root_path, fields, &|dup_a, dup_b| {
        format!(
            "fields `{}` and `{}` have identical name encoding",
            dup_a, dup_b
        )
    })
}

/// Produces a code which statically asserts that there's no duplicates in field name encodings
///
/// Takes a `root_path` (path to `udigest` lib), and list of fields
fn assert_no_duplicates_in_enum_variant_field_names(
    root_path: &attrs::RootPath,
    variant_name: &str,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    assert_no_duplicates_in_field_names(root_path, fields, &|dup_a, dup_b| {
        format!(
            "enum variant `{}` has fields `{}` and `{}` that have identical name encoding",
            variant_name, dup_a, dup_b
        )
    })
}

/// Produces a code which statically asserts that there's no duplicates in field name encodings
///
/// Takes a `root_path` (path to `udigest` lib), and list of fields
fn assert_no_duplicates_in_field_names(
    root_path: &attrs::RootPath,
    fields: &[Field],
    error_msg: &dyn Fn(&str, &str) -> String,
) -> proc_macro2::TokenStream {
    if fields.iter().all(|f| f.attrs.rename.is_none()) {
        // if no field was renamed, the check is not necessary: compiler won't allow
        // fields with the same names
        return proc_macro2::TokenStream::new();
    }

    // extract relevant info about fields
    let fields = fields
        .iter()
        .map(|f| {
            // span associated with the field
            let span = f.span;
            // name of the field as appears in field definition
            let name = f.stringify_field_name();
            // name of the field as appears in encoding
            let encoding = f
                .attrs
                .rename
                .as_ref()
                .map(|attrs::Rename { value, .. }| quote_spanned!(span => #value))
                .unwrap_or_else(|| quote_spanned!(span => #name));
            (span, name, encoding)
        })
        .collect::<Vec<_>>();

    assert_no_duplicates(root_path, &fields, error_msg)
}

fn assert_no_duplicates_in_enum_variant_names(
    root_path: &attrs::RootPath,
    enum_variants: &[Variant],
) -> proc_macro2::TokenStream {
    if enum_variants.iter().all(|v| v.attrs.rename.is_none()) {
        // if no variant was renamed, the check is not necessary: compiler won't allow
        // variants with the same name
        return proc_macro2::TokenStream::new();
    }

    let variants = enum_variants
        .iter()
        .map(|v| {
            let span = v.name.span();
            let name = v.name.to_string();
            let encoding = v
                .attrs
                .rename
                .as_ref()
                .map(|attrs::Rename { value, .. }| quote_spanned!(span => #value))
                .unwrap_or_else(|| quote_spanned!(span => #name));
            (span, name, encoding)
        })
        .collect::<Vec<_>>();

    assert_no_duplicates(root_path, &variants, &|dup_a, dup_b| {
        format!("variants `{dup_a}` and `{dup_b}` have identical encoding")
    })
}

/// Produces a code which statically asserts that there's no duplicates in statically-defined
/// encodings (of field names, variant names, etc.)
///
/// Takes a `root_path` (path to `udigest` lib), set of `[(span, name, encoding)]`, and an error message
/// formatter, and produces the code that statically asserts that there's no duplicates in this set of
/// `encoding`.
///
/// `span` from the `set` is only used to produce a hint for compiler so it could identify a source of error
/// more precisely (but compiler seems to ignore it for whatever reason at time of writing).
///
/// `name` from the `set` is only used to get an error message, by providing it to `error_msg` lambda.
fn assert_no_duplicates(
    root_path: &attrs::RootPath,
    set: &[(proc_macro2::Span, String, proc_macro2::TokenStream)],
    error_msg: &dyn Fn(&str, &str) -> String,
) -> proc_macro2::TokenStream {
    // for each (unordered) pair from the set, assert that their encoding isn't equal
    let pairs = set
        .iter()
        .enumerate()
        .flat_map(|(i, (span, a_name, a_name_encoding))| {
            set[i + 1..]
                .iter()
                .map(move |(_, b_name, b_name_encoding)| {
                    (
                        *span,
                        (a_name.clone(), a_name_encoding.clone()),
                        (b_name, b_name_encoding),
                    )
                })
        });
    let assertions = pairs.map(
        |(span, (a_name, a_name_encoding), (b_name, b_name_encoding))| {
            let error_msg = error_msg(&a_name, b_name);
            quote_spanned! {span => {
                #[allow(non_upper_case_globals, dead_code)]
                const has_unique_encoding: () = {
                    if #root_path::const_eq!(
                        #a_name_encoding,
                        #b_name_encoding,
                    ) {
                        panic!(#error_msg);
                    }
                };
            }}
        },
    );
    quote!(#(#assertions)*)
}

struct ContainerAttrs {
    root: Option<attrs::Root>,
    tag: Option<attrs::Tag>,
    bound: Option<attrs::Bound>,
    inferred_trait_path: attrs::RootPath,
    inferred_root_path: attrs::RootPath,
}

impl ContainerAttrs {
    pub fn new(inferred_paths: InferredPaths) -> Self {
        Self {
            root: None,
            tag: None,
            bound: None,
            inferred_trait_path: inferred_paths.inferred_trait_path,
            inferred_root_path: inferred_paths.inferred_root_path,
        }
    }

    pub fn get_root_path(&self) -> attrs::RootPath {
        self.root
            .as_ref()
            .map(|root| root.path.clone())
            .unwrap_or_else(|| self.inferred_root_path.clone())
    }

    pub fn get_trait_path(&self) -> attrs::RootPath {
        self.root
            .as_ref()
            .map(|root| root_path_to_trait_path(&root.path))
            .unwrap_or_else(|| self.inferred_trait_path.clone())
    }
}

struct InferredPaths {
    inferred_root_path: attrs::RootPath,
    inferred_trait_path: attrs::RootPath,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_trait_path_from_derive_supports_qualified_path() {
        let input: syn::DeriveInput = syn::parse_quote! {
            #[derive(Clone, udigest_encoding::Digestable)]
            struct Demo;
        };

        let trait_path = infer_trait_path_from_derive(&input).unwrap();
        assert_eq!(trait_path.segments.len(), 2);
        assert_eq!(trait_path.segments[0].ident, "udigest_encoding");
        assert_eq!(trait_path.segments[1].ident, "Digestable");
    }

    #[test]
    fn infer_trait_path_from_derive_supports_unqualified_path() {
        let input: syn::DeriveInput = syn::parse_quote! {
            #[derive(Digestable)]
            struct Demo;
        };

        let trait_path = infer_trait_path_from_derive(&input).unwrap();
        assert_eq!(trait_path.segments.len(), 1);
        assert_eq!(trait_path.segments[0].ident, "Digestable");
    }

    #[test]
    fn derive_trait_path_to_root_path_handles_qualified_path() {
        let path: syn::Path = syn::parse_quote!(::udigest::Digestable);
        let root = derive_trait_path_to_root_path(&path).unwrap();
        assert_eq!(root.segments.len(), 1);
        assert_eq!(root.segments[0].ident, "udigest");
    }

    #[test]
    fn derive_trait_path_to_root_path_handles_unqualified_path() {
        let path: syn::Path = syn::parse_quote!(Digestable);
        assert!(derive_trait_path_to_root_path(&path).is_none());
    }
}

#[derive(Default)]
struct FieldAttrs {
    as_bytes: Option<attrs::AsBytes>,
    skip: Option<attrs::Skip>,
    rename: Option<attrs::Rename>,
    with: Option<attrs::With>,
    as_: Option<attrs::As>,
}

#[derive(Default)]
struct EnumVariantAttrs {
    rename: Option<attrs::Rename>,
}

struct Field {
    span: proc_macro2::Span,
    attrs: FieldAttrs,
    mem: syn::Member,
    ty: syn::Type,
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
    attrs: EnumVariantAttrs,
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
