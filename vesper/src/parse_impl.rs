use twilight_model::channel::Attachment;
use twilight_model::guild::Role;
use twilight_model::user::User;
use crate::prelude::*;
use crate::twilight_exports::*;

const NUMBER_MAX_VALUE: i64 = 9007199254740991;

pub(crate) fn error(type_name: &str, required: bool, why: &str) -> ParseError {
    ParseError::Parsing {
        argument_name: String::new(),
        required,
        argument_type: type_name.to_string(),
        error: why.to_string()
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for String {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::String(s)) = value {
            return Ok(s.to_owned());
        }
        Err(error("String", true, "String expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::String
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for i64 {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::Integer(i)) = value {
            return Ok(*i);
        }
        Err(error("i64", true, "Integer expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::Integer
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for u64 {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::Integer(i)) = value {
            if *i < 0 {
                return Err(error("u64", true, "Input out of range"))
            }
            return Ok(*i as u64);
        }
        Err(error("Integer", true, "Integer expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::Integer
    }

    fn modify_option(option: &mut CommandOption) {
        use twilight_model::application::command::CommandOptionValue;
        option.min_value = Some(CommandOptionValue::Integer(0));
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for f64 {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::Number(i)) = value {
            return Ok(*i);
        }
        Err(error("f64", true, "Number expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::Number
    }

    fn modify_option(option: &mut CommandOption) {
        use twilight_model::application::command::CommandOptionValue;
        option.min_value = Some(CommandOptionValue::Number(f64::MIN));
        option.max_value = Some(CommandOptionValue::Number(f64::MAX));
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for f32 {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::Number(i)) = value {
            if *i > f32::MAX as f64 || *i < f32::MIN as f64 {
                return Err(error("f32", true, "Input out of range"))
            }
            return Ok(*i as f32);
        }
        Err(error("f32", true, "Number expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::Number
    }

    fn modify_option(option: &mut CommandOption) {
        use twilight_model::application::command::CommandOptionValue;
        option.max_value = Some(CommandOptionValue::Number(f32::MAX as f64));
        option.min_value = Some(CommandOptionValue::Number(f32::MIN as f64));
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for bool {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::Boolean(i)) = value {
            return Ok(*i);
        }
        Err(error("Boolean", true, "Boolean expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::Boolean
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for Id<AttachmentMarker> {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::Attachment(attachment)) = value {
            return Ok(*attachment);
        }

        Err(error("Attachment id", true, "Attachment expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::Attachment
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for Attachment {
    async fn parse(
        http_client: &WrappedClient,
        data: &T,
        value: Option<&CommandOptionValue>,
        resolved: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        let id = <Id<AttachmentMarker> as Parse<T>>::parse(http_client, data, value, None).await?;

        resolved.map(|item| item.attachments.remove(&id))
            .flatten()
            .ok_or_else(|| error("Attachment", true, "Attachment expected"))
    }

    fn kind() -> CommandOptionType {
        <Id<AttachmentMarker> as Parse<T>>::kind()
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for Id<ChannelMarker> {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::Channel(channel)) = value {
            return Ok(*channel);
        }

        Err(error("Channel id", true, "Channel expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::Channel
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for Id<UserMarker> {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::User(user)) = value {
            return Ok(*user);
        }

        Err(error("User id", true, "User expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::User
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for User {
    async fn parse(
        http_client: &WrappedClient,
        data: &T,
        value: Option<&CommandOptionValue>,
        resolved: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        let id = <Id<UserMarker> as Parse<T>>::parse(http_client, data, value, None).await?;

        resolved.map(|items| items.users.remove(&id))
            .flatten()
            .ok_or_else(|| error("User", true, "User expected"))
    }

    fn kind() -> CommandOptionType {
        <Id<UserMarker> as Parse<T>>::kind()
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for Id<RoleMarker> {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::Role(role)) = value {
            return Ok(*role);
        }

        Err(error("Role id", true, "Role expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::Role
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for Role {
    async fn parse(
        http_client: &WrappedClient,
        data: &T,
        value: Option<&CommandOptionValue>,
        resolved: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        let id = <Id<RoleMarker> as Parse<T>>::parse(http_client, data, value, None).await?;

        resolved.map(|items| items.roles.remove(&id))
            .flatten()
            .ok_or_else(|| error("Role", true, "Role expected"))
    }

    fn kind() -> CommandOptionType {
        <Id<RoleMarker> as Parse<T>>::kind()
    }
}

#[async_trait]
impl<T: Send + Sync> Parse<T> for Id<GenericMarker> {
    async fn parse(
        _: &WrappedClient,
        _: &T,
        value: Option<&CommandOptionValue>,
        _: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        if let Some(CommandOptionValue::Mentionable(id)) = value {
            return Ok(*id);
        }

        Err(error("Id", true, "Mentionable expected"))
    }

    fn kind() -> CommandOptionType {
        CommandOptionType::Mentionable
    }
}

#[async_trait]
impl<T: Parse<E>, E: Send + Sync> Parse<E> for Option<T> {
    async fn parse(
        http_client: &WrappedClient,
        data: &E,
        value: Option<&CommandOptionValue>,
        resolved: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        match T::parse(http_client, data, value, resolved).await {
            Ok(parsed) => Ok(Some(parsed)),
            Err(mut why) => {
                if value.is_some() {
                    if let ParseError::Parsing {required, ..} = &mut why {
                        *required = false;
                    }

                    Err(why)
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn kind() -> CommandOptionType {
        T::kind()
    }

    fn required() -> bool {
        false
    }

    fn choices() -> Option<Vec<CommandOptionChoice>> {
        T::choices()
    }

    fn modify_option(option: &mut CommandOption) {
        T::modify_option(option)
    }
}

#[async_trait]
impl<T, E, C> Parse<C> for Result<T, E>
where
    T: Parse<C>,
    E: From<ParseError>,
    C: Send + Sync,
{
    async fn parse(
        http_client: &WrappedClient,
        data: &C,
        value: Option<&CommandOptionValue>,
        resolved: Option<&mut InteractionDataResolved>
    ) -> Result<Self, ParseError> {
        // as we want to return the error if occurs, we'll map the error and always return Ok
        Ok(T::parse(http_client, data, value, resolved).await.map_err(From::from))
    }

    fn kind() -> CommandOptionType {
        T::kind()
    }

    fn required() -> bool {
        T::required()
    }

    fn choices() -> Option<Vec<CommandOptionChoice>> {
        T::choices()
    }

    fn modify_option(option: &mut CommandOption) {
        T::modify_option(option)
    }
}

macro_rules! impl_derived_parse {
    ($([$($derived:ty),+] from $prim:ty),* $(,)?) => {
        $($(
            #[async_trait]
            impl<T: Send + Sync> Parse<T> for $derived {
                async fn parse(
                    http_client: &WrappedClient,
                    data: &T,
                    value: Option<&CommandOptionValue>,
                    resolved: Option<&mut InteractionDataResolved>
                ) -> Result<Self, ParseError> {
                    let p = <$prim>::parse(http_client, data, value, resolved).await?;

                    if p > <$derived>::MAX as $prim {
                        Err(error(
                            stringify!($derived),
                            true,
                            concat!(
                                "Failed to parse to ",
                                stringify!($derived),
                                ": the value is greater than ",
                                stringify!($derived),
                                "'s ",
                                "range of values"
                            )
                        ))
                    } else if p < <$derived>::MIN as $prim {
                        Err(error(
                            stringify!($derived),
                            true,
                            concat!(
                                "Failed to parse to ",
                                stringify!($derived),
                                ": the value is less than ",
                                stringify!($derived),
                                "'s ",
                                "range of values"
                            )
                        ))
                    } else {
                        Ok(p as $derived)
                    }
                }

                fn kind() -> CommandOptionType {
                    <$prim as Parse<T>>::kind()
                }

                fn modify_option(option: &mut CommandOption) {
                    use twilight_model::application::command::CommandOptionValue;
                    option.max_value = Some(CommandOptionValue::Integer({
                        if <$derived>::MAX as i64 > NUMBER_MAX_VALUE {
                            NUMBER_MAX_VALUE
                        } else {
                            <$derived>::MAX as i64
                        }
                    }));

                    option.min_value = Some(CommandOptionValue::Integer(<$derived>::MIN as i64));
                }
            }
        )*)*
    };
}

impl_derived_parse! {
    [i8, i16, i32, isize] from i64,
    [u8, u16, u32, usize] from u64,
}
