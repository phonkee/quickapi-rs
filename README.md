# QuickAPI

QuickAPI is rust package for building restful APIs for sea ORM models and axum framework.

# Status

> [!WARNING]  
> This project is in early development stage. API may change without notice.

# Features

This package provides easy way to create RESTful API endpoints for your sea ORM models using axum framework. 
It not only allows simple CRUD operations, it also provides a way how to add when conditions to whole views.

# Design

Each "view" provides single type of operation on single entity type.
Operations are:

- **List** - list all entities of given type
- **Create** - create new entity of given type
- **Detail** - detail view of single entity
- **Update** - update single entity
- **Delete** - delete single entity

### Filter

Each one of these operations (except of create) have ability to filter select query.
You can provide multiple filters that resemble to axum handlers, where first argument is Select on given Model Entity,
And other arguments are `axum::extract::FromRequest` types that are used to filter the query.

### When 

You can also provide "when" conditions that clone given view and add you ability to change it.
When accepts function that resembles to axum handlers, when arguments are `axum::extract::FromRequest` 
and you need to return `Result<()>` when this condition is met or 
Result<(), quickapi_when::Error> when it is not met.
If you return `NoMatch` quickapi will continue to evaluate next when condition.


# Views

### List View

List view is used to list all entities of given type. You can filter them, and use predefined filter basic blocks such as :

- Paginator -  This filter allows you to paginate results by providing `page` and `per_page` query parameters.

```rust
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct QueryFormat {
    format: Option<String>,
}

// filter_search_query filters the search query
pub async fn filter_search_query_username(
    query: Select<entity::User>,
    search: Query<QuerySearch>,
) -> Result<Select<entity::User>, quickapi_filter::Error> {
    // if query is present, filter by username
    Ok(if let Some(s) = search.0.query {
        query.filter(entity::user::Column::Username.contains(s))
    } else {
        query
    })
}

// add list view for User entity
let router = api
    .list::<entity::User>("/api/user")?
    .with_filter(Paginator::default())
    .with_filter(filter_search_query_username)
    .with_serializer::<serializers::UsernameOnly>()
    .wrap_result_key("users")
    .when(async move |search: Query<QuerySearch>| {
        if search.query.is_some() {
            Ok(())
        } else {
            Err(quickapi_when::Error::NoMatch)
        }
    }, |v| {
        // change serializer for this condition
        Ok(v.with_serializer::<serializers::SimpleUser>())
    })?.register_router(router)?;
```

### Detail View

Detail view is used to get single entity by single field, usually by primary key.
You can filter it, use when conditions and serializers.:

```rust
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct QueryFormat {
    format: Option<String>,
}

/// when_condition is a condition that will be checked before applying the view
pub async fn when_condition_format(_x: Query<QueryFormat>) -> Result<(), quickapi_when::Error> {
    match &_x.format {
        Some(format) if format == "full" => Ok(()),
        _ => Err(quickapi_when::Error::NoMatch),
    }
}

// add detail view for User entity
let router = api
    .detail::<entity::User>("/api/user/{id}", PrimaryKey::Path("id".into()))?
    .with_serializer::<serializers::UsernameOnly>()
    .wrap_result_key("user")
    .when(when_condition_format, |v| {
        Ok(v.with_serializer::<serializers::SimpleUser>())
    })?.register_router(router)?;

```

### Create View

Create view is used to create new entity of given type. It accepts JSON body with entity data and returns created entity.

> [!WARNING]  
> Not implemented yet. Work in progress.


### Update View

Update view is used to update single entity by single field, usually by primary key.

> [!WARNING]  
> Not implemented yet. Work in progress.

### Delete View

Delete view is used to delete single entity by single field, usually by primary key.

> [!WARNING]  
> Not implemented yet. Work in progress.


# Author

Peter Vrba <phonkee@phonkee.eu>
