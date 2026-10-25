//! Compiled counterparts of `blog/2026-05-10-v0.8.0-release.md`.
//! Modules follow the walkthrough. Old-version migration snippets are historical input;
//! the missing-height program is a compile-fail fixture under `tests/compile_fail/blog/`.
#![allow(dead_code, unused_variables)]

mod diagnosing_wiring_with_cargo_cgp {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator {
        fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            width * height
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    delegate_and_check_components! {
        Rectangle {
            AreaCalculatorComponent: RectangleArea,
        }
    }
    #[test]
    fn repaired_rectangle_has_an_area() {
        assert_eq!(
            Rectangle {
                width: 3.0,
                height: 4.0
            }
            .area(),
            12.0
        );
    }
}

mod domain {
    pub struct Email(pub String);

    pub struct User;
    pub struct UserData;
    pub struct UserId;

    pub struct Post;
    pub struct PostId;

    pub enum Error {
        InvalidUsername,
        InvalidMessage,
    }

    pub struct PostgresDb;

    #[derive(PartialOrd, PartialEq)]
    pub struct Probability(pub f64);

    impl Probability {
        pub const fn new(value: f64) -> Self {
            Self(value)
        }
    }
}

mod start_with_database_only_managers {
    use super::domain::*;
    use cgp::prelude::*;

    #[cgp_component(UserManager)]
    pub trait CanManageUser {
        fn create_user(&self, username: &str, email: &Email) -> Result<User, Error>;

        fn get_user(&self, user_id: &UserId) -> Result<User, Error>;

        fn update_user_data(&self, user_id: &UserId, user_data: &UserData) -> Result<(), Error>;
    }

    #[cgp_component(PostManager)]
    pub trait CanManagePost {
        fn create_post(&self, title: &str, content: &str) -> Result<Post, Error>;

        fn get_post(&self, post_id: &PostId) -> Result<Post, Error>;

        fn update_post(&self, post_id: &PostId, content: &str) -> Result<(), Error>;

        fn delete_post(&self, post_id: &PostId) -> Result<(), Error>;
    }

    #[cgp_impl(new PostgresUserManager)]
    impl UserManager {
        fn create_user(
            &self,
            #[implicit] database: &PostgresDb,
            username: &str,
            email: &Email,
        ) -> Result<User, Error> {
            todo!() // The page elides the database query.
        }

        fn get_user(
            &self,
            #[implicit] database: &PostgresDb,
            user_id: &UserId,
        ) -> Result<User, Error> {
            todo!() // The page elides the database query.
        }

        fn update_user_data(
            &self,
            #[implicit] database: &PostgresDb,
            user_id: &UserId,
            user_data: &UserData,
        ) -> Result<(), Error> {
            todo!() // The page elides the database query.
        }
    }

    #[cgp_impl(new PostgresPostManager)]
    impl PostManager {
        fn create_post(
            &self,
            #[implicit] database: &PostgresDb,
            title: &str,
            content: &str,
        ) -> Result<Post, Error> {
            todo!() // The page elides the database query.
        }

        fn get_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
        ) -> Result<Post, Error> {
            todo!() // The page elides the database query.
        }

        fn update_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
            content: &str,
        ) -> Result<(), Error> {
            todo!() // The page elides the database query.
        }

        fn delete_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
        ) -> Result<(), Error> {
            todo!() // The page elides the database query.
        }
    }

    #[derive(HasField)]
    pub struct ProductionApp {
        pub database: PostgresDb,
    }

    delegate_and_check_components! {
        ProductionApp {
            UserManagerComponent: PostgresUserManager,
            PostManagerComponent: PostgresPostManager,
        }
    }
}

mod start_with_a_manager_and_add_dependencies {
    use cgp::prelude::*;

    use super::domain::*;

    #[cgp_component(UserManager)]
    pub trait CanManageUser {
        fn create_user(&self, username: &str, email: &Email) -> Result<User, Error>;

        fn get_user(&self, user_id: &UserId) -> Result<User, Error>;

        fn update_user_data(&self, user_id: &UserId, user_data: &UserData) -> Result<(), Error>;
    }

    #[cgp_component(PostManager)]
    pub trait CanManagePost {
        fn create_post(&self, title: &str, content: &str) -> Result<Post, Error>;

        fn get_post(&self, post_id: &PostId) -> Result<Post, Error>;

        fn update_post(&self, post_id: &PostId, content: &str) -> Result<(), Error>;

        fn delete_post(&self, post_id: &PostId) -> Result<(), Error>;
    }

    #[cgp_component(UsernameCensor)]
    pub trait CanCensorUsername {
        fn username_is_censored(&self, username: &str) -> Probability;
    }

    #[cgp_component(SpamMessageDetector)]
    pub trait CanDetectSpamMessage {
        fn message_is_spam(&self, message: &str) -> Probability;
    }

    #[cgp_impl(new PostgresUserManager)]
    #[uses(CanCensorUsername)]
    impl UserManager {
        fn create_user(
            &self,
            #[implicit] database: &PostgresDb,
            username: &str,
            email: &Email,
        ) -> Result<User, Error> {
            if self.username_is_censored(username) > Probability::new(0.8) {
                return Err(Error::InvalidUsername);
            }

            todo!() // The page elides the database/filter body.
        }

        fn get_user(
            &self,
            #[implicit] database: &PostgresDb,
            user_id: &UserId,
        ) -> Result<User, Error> {
            todo!() // The page elides the database/filter body.
        }

        fn update_user_data(
            &self,
            #[implicit] database: &PostgresDb,
            user_id: &UserId,
            user_data: &UserData,
        ) -> Result<(), Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new PostgresPostManager)]
    #[uses(CanDetectSpamMessage)]
    impl PostManager {
        fn create_post(
            &self,
            #[implicit] database: &PostgresDb,
            title: &str,
            content: &str,
        ) -> Result<Post, Error> {
            if self.message_is_spam(content) > Probability::new(0.8) {
                return Err(Error::InvalidMessage);
            }

            todo!() // The page elides the database/filter body.
        }

        fn get_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
        ) -> Result<Post, Error> {
            todo!() // The page elides the database/filter body.
        }

        fn update_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
            content: &str,
        ) -> Result<(), Error> {
            todo!() // The page elides the database/filter body.
        }

        fn delete_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
        ) -> Result<(), Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new DummyUserCensor)]
    impl UsernameCensor {
        fn username_is_censored(&self, username: &str) -> Probability {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new DummySpamMessageDetector)]
    impl SpamMessageDetector {
        fn message_is_spam(&self, message: &str) -> Probability {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new AiUserCensor)]
    impl UsernameCensor {
        fn username_is_censored(&self, username: &str) -> Probability {
            todo!() // The page omits the filter implementation.
        }
    }
    #[cgp_impl(new AiSpamMessageDetector)]
    impl SpamMessageDetector {
        fn message_is_spam(&self, message: &str) -> Probability {
            todo!() // The page omits the filter implementation.
        }
    }
    #[derive(HasField)]
    pub struct ProductionApp {
        pub database: PostgresDb,
    }

    delegate_components! {
        ProductionApp {
            UserManagerComponent: PostgresUserManager,
            PostManagerComponent: PostgresPostManager,
            UsernameCensorComponent: AiUserCensor,
            SpamMessageDetectorComponent: AiSpamMessageDetector,
        }
    }

    #[derive(HasField)]
    pub struct TestApp {
        pub database: PostgresDb,
    }

    delegate_components! {
        TestApp {
            UserManagerComponent: PostgresUserManager,
            PostManagerComponent: PostgresPostManager,
            UsernameCensorComponent: DummyUserCensor,
            SpamMessageDetectorComponent: DummySpamMessageDetector,
        }
    }
    check_components! { ProductionApp { UserManagerComponent, PostManagerComponent, UsernameCensorComponent, SpamMessageDetectorComponent, } }
    check_components! { TestApp { UserManagerComponent, PostManagerComponent, UsernameCensorComponent, SpamMessageDetectorComponent, } }
}

mod separate_filtering_from_database_access {
    use cgp::prelude::*;

    use super::domain::*;

    #[cgp_component(UserCreator)]
    pub trait CanCreateUser {
        fn create_user(&self, username: &str, email: &Email) -> Result<User, Error>;
    }

    #[cgp_component(UserGetter)]
    pub trait CanGetUser {
        fn get_user(&self, user_id: &UserId) -> Result<User, Error>;
    }

    #[cgp_component(UserUpdater)]
    pub trait CanUpdateUser {
        fn update_user_data(&self, user_id: &UserId, user_data: &UserData) -> Result<(), Error>;
    }

    #[cgp_component(PostCreator)]
    pub trait CanCreatePost {
        fn create_post(&self, title: &str, content: &str) -> Result<Post, Error>;
    }

    #[cgp_component(PostGetter)]
    pub trait CanGetPost {
        fn get_post(&self, post_id: &PostId) -> Result<Post, Error>;
    }

    #[cgp_component(PostUpdater)]
    pub trait CanUpdatePost {
        fn update_post(&self, post_id: &PostId, content: &str) -> Result<(), Error>;
    }

    #[cgp_component(PostDeleter)]
    pub trait CanDeletePost {
        fn delete_post(&self, post_id: &PostId) -> Result<(), Error>;
    }

    #[cgp_component(UsernameCensor)]
    pub trait CanCensorUsername {
        fn username_is_censored(&self, username: &str) -> Probability;
    }

    #[cgp_component(SpamMessageDetector)]
    pub trait CanDetectSpamMessage {
        fn message_is_spam(&self, message: &str) -> Probability;
    }

    #[cgp_impl(new FilterCensoredUsername<InnerCreator>)]
    #[uses(CanCensorUsername)]
    #[use_provider(InnerCreator: UserCreator)]
    impl<InnerCreator> UserCreator {
        fn create_user(&self, username: &str, email: &Email) -> Result<User, Error> {
            if self.username_is_censored(username) > Probability::new(0.8) {
                return Err(Error::InvalidUsername);
            }

            InnerCreator::create_user(self, username, email)
        }
    }

    #[cgp_impl(new CreateUserWithPostgres)]
    impl UserCreator {
        fn create_user(
            &self,
            #[implicit] database: &PostgresDb,
            username: &str,
            email: &Email,
        ) -> Result<User, Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new GetUserWithPostgres)]
    impl UserGetter {
        fn get_user(
            &self,
            #[implicit] database: &PostgresDb,
            user_id: &UserId,
        ) -> Result<User, Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new UpdateUserWithPostgres)]
    impl UserUpdater {
        fn update_user_data(
            &self,
            #[implicit] database: &PostgresDb,
            user_id: &UserId,
            user_data: &UserData,
        ) -> Result<(), Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new FilterSpamMessage<InnerCreator>)]
    #[uses(CanDetectSpamMessage)]
    #[use_provider(InnerCreator: PostCreator)]
    impl<InnerCreator> PostCreator {
        fn create_post(&self, title: &str, content: &str) -> Result<Post, Error> {
            if self.message_is_spam(content) > Probability::new(0.8) {
                return Err(Error::InvalidMessage);
            }

            InnerCreator::create_post(self, title, content)
        }
    }

    #[cgp_impl(new CreatePostWithPostgres)]
    impl PostCreator {
        fn create_post(
            &self,
            #[implicit] database: &PostgresDb,
            title: &str,
            content: &str,
        ) -> Result<Post, Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new GetPostWithPostgres)]
    impl PostGetter {
        fn get_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
        ) -> Result<Post, Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new UpdatePostWithPostgres)]
    impl PostUpdater {
        fn update_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
            content: &str,
        ) -> Result<(), Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new DeletePostWithPostgres)]
    impl PostDeleter {
        fn delete_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
        ) -> Result<(), Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new DummyUserCensor)]
    impl UsernameCensor {
        fn username_is_censored(&self, username: &str) -> Probability {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new AiUserCensor)]
    impl UsernameCensor {
        fn username_is_censored(&self, username: &str) -> Probability {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new DummySpamMessageDetector)]
    impl SpamMessageDetector {
        fn message_is_spam(&self, message: &str) -> Probability {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new AiSpamMessageDetector)]
    impl SpamMessageDetector {
        fn message_is_spam(&self, message: &str) -> Probability {
            todo!() // The page elides the database/filter body.
        }
    }
}

mod split_independently_reusable_operations {
    use super::domain::*;
    use super::separate_filtering_from_database_access::*;
    use cgp::prelude::*;
    #[cgp_impl(new CreateUserWithPostgres)]
    #[uses(CanCensorUsername)]
    impl UserCreator {
        fn create_user(
            &self,
            #[implicit] database: &PostgresDb,
            username: &str,
            email: &Email,
        ) -> Result<User, Error> {
            if self.username_is_censored(username) > Probability::new(0.8) {
                return Err(Error::InvalidUsername);
            }

            todo!() // The page elides the database query.
        }
    }

    #[cgp_impl(new GetUserWithPostgres)]
    impl UserGetter {
        fn get_user(
            &self,
            #[implicit] database: &PostgresDb,
            user_id: &UserId,
        ) -> Result<User, Error> {
            todo!() // The page elides the database query.
        }
    }
    #[derive(HasField)]
    pub struct ProductionApp {
        pub database: PostgresDb,
    }
    delegate_components! {
        ProductionApp {
            UserCreatorComponent:
                FilterCensoredUsername<CreateUserWithPostgres>,
            UserGetterComponent:
                GetUserWithPostgres,
            UserUpdaterComponent:
                UpdateUserWithPostgres,
            PostCreatorComponent:
                FilterSpamMessage<CreatePostWithPostgres>,
            PostGetterComponent:
                GetPostWithPostgres,
            PostUpdaterComponent:
                UpdatePostWithPostgres,
            PostDeleterComponent:
                DeletePostWithPostgres,
            UsernameCensorComponent:
                AiUserCensor,
            SpamMessageDetectorComponent:
                AiSpamMessageDetector,
        }
    }
    check_components! { ProductionApp { UserCreatorComponent, UserGetterComponent, UserUpdaterComponent, PostCreatorComponent, PostGetterComponent, PostUpdaterComponent, PostDeleterComponent, UsernameCensorComponent, SpamMessageDetectorComponent, } }
}

mod count_the_cost_of_the_separation {
    use super::domain::*;
    use super::separate_filtering_from_database_access::*;
    use cgp::prelude::*;
    #[derive(HasField)]
    pub struct ProductionApp {
        pub database: PostgresDb,
    }
    delegate_components! {
        ProductionApp {
            UserCreatorComponent:
                FilterCensoredUsername<CreateUserWithPostgres>,
            UserGetterComponent:
                GetUserWithPostgres,
            UserUpdaterComponent:
                UpdateUserWithPostgres,
            PostCreatorComponent:
                FilterSpamMessage<CreatePostWithPostgres>,
            PostGetterComponent:
                GetPostWithPostgres,
            PostUpdaterComponent:
                UpdatePostWithPostgres,
            PostDeleterComponent:
                DeletePostWithPostgres,
            UsernameCensorComponent:
                AiUserCensor,
            SpamMessageDetectorComponent:
                AiSpamMessageDetector,
        }
    }
    check_components! { ProductionApp { UserCreatorComponent, UserGetterComponent, UserUpdaterComponent, PostCreatorComponent, PostGetterComponent, PostUpdaterComponent, PostDeleterComponent, UsernameCensorComponent, SpamMessageDetectorComponent, } }
}

mod group_providers_into_bundles {
    use super::domain::*;
    use super::separate_filtering_from_database_access::*;
    use cgp::prelude::*;
    delegate_components! {
        new PostgresUserComponents {
            UserCreatorComponent:
                FilterCensoredUsername<CreateUserWithPostgres>,
            UserGetterComponent:
                GetUserWithPostgres,
            UserUpdaterComponent:
                UpdateUserWithPostgres,
        }
    }

    delegate_components! {
        new PostgresPostComponents {
            PostCreatorComponent:
                FilterSpamMessage<CreatePostWithPostgres>,
            PostGetterComponent:
                GetPostWithPostgres,
            PostUpdaterComponent:
                UpdatePostWithPostgres,
            PostDeleterComponent:
                DeletePostWithPostgres,
        }
    }

    delegate_components! {
        new AiContentFilterComponents {
            UsernameCensorComponent:
                AiUserCensor,
            SpamMessageDetectorComponent:
                AiSpamMessageDetector,
        }
    }

    #[derive(HasField)]
    pub struct ProductionApp {
        pub database: PostgresDb,
    }
    delegate_components! {
        ProductionApp {
            [
                UserCreatorComponent,
                UserGetterComponent,
                UserUpdaterComponent,
            ]:
                PostgresUserComponents,
            [
                PostCreatorComponent,
                PostGetterComponent,
                PostUpdaterComponent,
                PostDeleterComponent,
            ]:
                PostgresPostComponents,
            [
                UsernameCensorComponent,
                SpamMessageDetectorComponent,
            ]:
                AiContentFilterComponents,
        }
    }
    check_components! { ProductionApp { UserCreatorComponent, UserGetterComponent, UserUpdaterComponent, PostCreatorComponent, PostGetterComponent, PostUpdaterComponent, PostDeleterComponent, UsernameCensorComponent, SpamMessageDetectorComponent, } }
}

mod give_component_groups_a_path {
    use cgp::prelude::*;

    use super::domain::*;

    #[cgp_component(UserCreator)]
    #[prefix(@app.core.user in DefaultNamespace)]
    pub trait CanCreateUser {
        fn create_user(&self, username: &str, email: &Email) -> Result<User, Error>;
    }

    #[cgp_component(UserGetter)]
    #[prefix(@app.core.user in DefaultNamespace)]
    pub trait CanGetUser {
        fn get_user(&self, user_id: &UserId) -> Result<User, Error>;
    }

    #[cgp_component(UserUpdater)]
    #[prefix(@app.core.user in DefaultNamespace)]
    pub trait CanUpdateUser {
        fn update_user_data(&self, user_id: &UserId, user_data: &UserData) -> Result<(), Error>;
    }

    #[cgp_component(PostCreator)]
    #[prefix(@app.core.post in DefaultNamespace)]
    pub trait CanCreatePost {
        fn create_post(&self, title: &str, content: &str) -> Result<Post, Error>;
    }

    #[cgp_component(PostGetter)]
    #[prefix(@app.core.post in DefaultNamespace)]
    pub trait CanGetPost {
        fn get_post(&self, post_id: &PostId) -> Result<Post, Error>;
    }

    #[cgp_component(PostUpdater)]
    #[prefix(@app.core.post in DefaultNamespace)]
    pub trait CanUpdatePost {
        fn update_post(&self, post_id: &PostId, content: &str) -> Result<(), Error>;
    }

    #[cgp_component(PostDeleter)]
    #[prefix(@app.core.post in DefaultNamespace)]
    pub trait CanDeletePost {
        fn delete_post(&self, post_id: &PostId) -> Result<(), Error>;
    }

    #[cgp_component(UsernameCensor)]
    #[prefix(@app.extra.content_filter in DefaultNamespace)]
    pub trait CanCensorUsername {
        fn username_is_censored(&self, username: &str) -> Probability;
    }

    #[cgp_component(SpamMessageDetector)]
    #[prefix(@app.extra.content_filter in DefaultNamespace)]
    pub trait CanDetectSpamMessage {
        fn message_is_spam(&self, message: &str) -> Probability;
    }

    #[cgp_impl(new FilterCensoredUsername<InnerCreator>)]
    #[uses(CanCensorUsername)]
    #[use_provider(InnerCreator: UserCreator)]
    impl<InnerCreator> UserCreator {
        fn create_user(&self, username: &str, email: &Email) -> Result<User, Error> {
            if self.username_is_censored(username) > Probability::new(0.8) {
                return Err(Error::InvalidUsername);
            }

            InnerCreator::create_user(self, username, email)
        }
    }

    #[cgp_impl(new CreateUserWithPostgres)]
    impl UserCreator {
        fn create_user(
            &self,
            #[implicit] database: &PostgresDb,
            username: &str,
            email: &Email,
        ) -> Result<User, Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new GetUserWithPostgres)]
    impl UserGetter {
        fn get_user(
            &self,
            #[implicit] database: &PostgresDb,
            user_id: &UserId,
        ) -> Result<User, Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new UpdateUserWithPostgres)]
    impl UserUpdater {
        fn update_user_data(
            &self,
            #[implicit] database: &PostgresDb,
            user_id: &UserId,
            user_data: &UserData,
        ) -> Result<(), Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new FilterSpamMessage<InnerCreator>)]
    #[uses(CanDetectSpamMessage)]
    #[use_provider(InnerCreator: PostCreator)]
    impl<InnerCreator> PostCreator {
        fn create_post(&self, title: &str, content: &str) -> Result<Post, Error> {
            if self.message_is_spam(content) > Probability::new(0.8) {
                return Err(Error::InvalidMessage);
            }

            InnerCreator::create_post(self, title, content)
        }
    }

    #[cgp_impl(new CreatePostWithPostgres)]
    impl PostCreator {
        fn create_post(
            &self,
            #[implicit] database: &PostgresDb,
            title: &str,
            content: &str,
        ) -> Result<Post, Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new GetPostWithPostgres)]
    impl PostGetter {
        fn get_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
        ) -> Result<Post, Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new UpdatePostWithPostgres)]
    impl PostUpdater {
        fn update_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
            content: &str,
        ) -> Result<(), Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new DeletePostWithPostgres)]
    impl PostDeleter {
        fn delete_post(
            &self,
            #[implicit] database: &PostgresDb,
            post_id: &PostId,
        ) -> Result<(), Error> {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new DummyUserCensor)]
    impl UsernameCensor {
        fn username_is_censored(&self, username: &str) -> Probability {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new AiUserCensor)]
    impl UsernameCensor {
        fn username_is_censored(&self, username: &str) -> Probability {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new DummySpamMessageDetector)]
    impl SpamMessageDetector {
        fn message_is_spam(&self, message: &str) -> Probability {
            todo!() // The page elides the database/filter body.
        }
    }

    #[cgp_impl(new AiSpamMessageDetector)]
    impl SpamMessageDetector {
        fn message_is_spam(&self, message: &str) -> Probability {
            todo!() // The page elides the database/filter body.
        }
    }

    delegate_components! {
        new PostgresUserComponents {
            UserCreatorComponent:
                FilterCensoredUsername<CreateUserWithPostgres>,
            UserGetterComponent:
                GetUserWithPostgres,
            UserUpdaterComponent:
                UpdateUserWithPostgres,
        }
    }

    delegate_components! {
        new PostgresPostComponents {
            PostCreatorComponent:
                FilterSpamMessage<CreatePostWithPostgres>,
            PostGetterComponent:
                GetPostWithPostgres,
            PostUpdaterComponent:
                UpdatePostWithPostgres,
            PostDeleterComponent:
                DeletePostWithPostgres,
        }
    }

    delegate_components! {
        new AiContentFilterComponents {
            UsernameCensorComponent:
                AiUserCensor,
            SpamMessageDetectorComponent:
                AiSpamMessageDetector,
        }
    }

    delegate_components! {
        new DummyContentFilterComponents {
            UsernameCensorComponent:
                DummyUserCensor,
            SpamMessageDetectorComponent:
                DummySpamMessageDetector,
        }
    }
}

mod forward_leaf_paths {
    use super::domain::*;
    use super::give_component_groups_a_path::*;
    use cgp::prelude::*;
    #[derive(HasField)]
    pub struct ProductionApp {
        pub database: PostgresDb,
    }
    delegate_components! {
        ProductionApp {
            namespace DefaultNamespace;

            @app.core.user: PostgresUserComponents,
            @app.core.post: PostgresPostComponents,
            @app.extra.content_filter: AiContentFilterComponents,
        }
    }
    check_components! {
        ProductionApp {
            UserCreatorComponent,
            UserGetterComponent,
            UserUpdaterComponent,
            PostCreatorComponent,
            PostGetterComponent,
            PostUpdaterComponent,
            PostDeleterComponent,
            UsernameCensorComponent,
            SpamMessageDetectorComponent,
        }
    }
}

mod group_paths_hierarchically_and_check_the_configuration {
    use super::domain::*;
    use super::give_component_groups_a_path::*;
    use cgp::prelude::*;
    delegate_components! {
        new PostgresCoreComponents {
            namespace DefaultNamespace;

            @app.core.user: PostgresUserComponents,
            @app.core.post: PostgresPostComponents,
            // other core components
        }
    }

    delegate_components! {
        new ProductionExtraComponents {
            namespace DefaultNamespace;

            @app.extra.content_filter: AiContentFilterComponents,
            // other extra components
        }
    }
    delegate_components! {
        new DummyExtraComponents {
            namespace DefaultNamespace;
            @app.extra.content_filter: DummyContentFilterComponents,
        }
    }
    #[derive(HasField)]
    pub struct ProductionApp {
        pub database: PostgresDb,
    }
    #[derive(HasField)]
    pub struct TestApp {
        pub database: PostgresDb,
    }
    delegate_components! {
        ProductionApp {
            namespace DefaultNamespace;

            @app.core: PostgresCoreComponents,
            @app.extra: ProductionExtraComponents,
        }
    }
    delegate_components! {
        TestApp {
            namespace DefaultNamespace;

            @app.core: PostgresCoreComponents,
            @app.extra: DummyExtraComponents,
        }
    }
    pub struct SqliteDb;
    #[derive(HasField)]
    pub struct LocalApp {
        pub database: SqliteDb,
    }
    #[cgp_impl(new CreateUserWithSqlite)]
    impl UserCreator {
        fn create_user(
            &self,
            #[implicit] database: &SqliteDb,
            username: &str,
            email: &Email,
        ) -> Result<User, Error> {
            todo!() // The page omits the hypothetical SQLite implementation.
        }
    }
    #[cgp_impl(new GetUserWithSqlite)]
    impl UserGetter {
        fn get_user(
            &self,
            #[implicit] database: &SqliteDb,
            user_id: &UserId,
        ) -> Result<User, Error> {
            todo!() // The page omits the hypothetical SQLite implementation.
        }
    }
    #[cgp_impl(new UpdateUserWithSqlite)]
    impl UserUpdater {
        fn update_user_data(
            &self,
            #[implicit] database: &SqliteDb,
            user_id: &UserId,
            user_data: &UserData,
        ) -> Result<(), Error> {
            todo!() // The page omits the hypothetical SQLite implementation.
        }
    }
    #[cgp_impl(new CreatePostWithSqlite)]
    impl PostCreator {
        fn create_post(
            &self,
            #[implicit] database: &SqliteDb,
            title: &str,
            content: &str,
        ) -> Result<Post, Error> {
            todo!() // The page omits the hypothetical SQLite implementation.
        }
    }
    #[cgp_impl(new GetPostWithSqlite)]
    impl PostGetter {
        fn get_post(
            &self,
            #[implicit] database: &SqliteDb,
            post_id: &PostId,
        ) -> Result<Post, Error> {
            todo!() // The page omits the hypothetical SQLite implementation.
        }
    }
    #[cgp_impl(new UpdatePostWithSqlite)]
    impl PostUpdater {
        fn update_post(
            &self,
            #[implicit] database: &SqliteDb,
            post_id: &PostId,
            content: &str,
        ) -> Result<(), Error> {
            todo!() // The page omits the hypothetical SQLite implementation.
        }
    }
    #[cgp_impl(new DeletePostWithSqlite)]
    impl PostDeleter {
        fn delete_post(
            &self,
            #[implicit] database: &SqliteDb,
            post_id: &PostId,
        ) -> Result<(), Error> {
            todo!() // The page omits the hypothetical SQLite implementation.
        }
    }
    delegate_components! {
        new SqliteCoreComponents {
            UserCreatorComponent: FilterCensoredUsername<CreateUserWithSqlite>,
            UserGetterComponent: GetUserWithSqlite,
            UserUpdaterComponent: UpdateUserWithSqlite,
            PostCreatorComponent: FilterSpamMessage<CreatePostWithSqlite>,
            PostGetterComponent: GetPostWithSqlite,
            PostUpdaterComponent: UpdatePostWithSqlite,
            PostDeleterComponent: DeletePostWithSqlite,
        }
    }
    delegate_components! {
        LocalApp {
            namespace DefaultNamespace;

            @app.core: SqliteCoreComponents,
            @app.extra: ProductionExtraComponents,
        }
    }
    check_components! {
        ProductionApp {
            UserCreatorComponent,
            UserGetterComponent,
            UserUpdaterComponent,
            PostCreatorComponent,
            PostGetterComponent,
            PostUpdaterComponent,
            PostDeleterComponent,
            UsernameCensorComponent,
            SpamMessageDetectorComponent,
        }
    }
    check_components! { TestApp { UserCreatorComponent, UserGetterComponent, UserUpdaterComponent, PostCreatorComponent, PostGetterComponent, PostUpdaterComponent, PostDeleterComponent, UsernameCensorComponent, SpamMessageDetectorComponent, } }
    check_components! { LocalApp { UserCreatorComponent, UserGetterComponent, UserUpdaterComponent, PostCreatorComponent, PostGetterComponent, PostUpdaterComponent, PostDeleterComponent, UsernameCensorComponent, SpamMessageDetectorComponent, } }
}

mod sharing_default_implementations {
    use super::domain::*;
    use super::give_component_groups_a_path::*;
    use cgp::prelude::*;
    cgp_namespace! {
        new DefaultAppComponents: DefaultNamespace
    }
    #[cgp_impl(new GetUserWithPostgres)]
    #[default_impl(@app.core.user.UserGetterComponent in DefaultAppComponents)]
    impl UserGetter {
        fn get_user(
            &self,
            #[implicit] database: &PostgresDb,
            user_id: &UserId,
        ) -> Result<User, Error> {
            todo!() // The page elides the database query.
        }
    }
    cgp_namespace! {
        DefaultAppComponents {
            @app.core.user.UserCreatorComponent:
                FilterCensoredUsername<CreateUserWithPostgres>,
        }
    }
    cgp_namespace! {
        DefaultAppComponents {
            @app.core.user.UserUpdaterComponent: UpdateUserWithPostgres,
            @app.core.post.PostCreatorComponent: FilterSpamMessage<CreatePostWithPostgres>,
            @app.core.post.PostGetterComponent: GetPostWithPostgres,
            @app.core.post.PostUpdaterComponent: UpdatePostWithPostgres,
            @app.core.post.PostDeleterComponent: DeletePostWithPostgres,
        }
    }
    type ContentFilterComponents = AiContentFilterComponents;

    #[derive(HasField)]
    pub struct ProductionApp {
        pub database: PostgresDb,
    }
    delegate_components! {
        ProductionApp {
            namespace DefaultAppComponents;

            @app.extra.content_filter: ContentFilterComponents,
        }
    }
    check_components! {
        ProductionApp {
            UserCreatorComponent,
            UserGetterComponent,
            UserUpdaterComponent,
            PostCreatorComponent,
            PostGetterComponent,
            PostUpdaterComponent,
            PostDeleterComponent,
            UsernameCensorComponent,
            SpamMessageDetectorComponent,
        }
    }
    #[derive(HasField)]
    pub struct TestApp {
        pub database: PostgresDb,
    }
    delegate_components! {
        TestApp {
            namespace DefaultAppComponents;
            @app.extra.content_filter: DummyContentFilterComponents,
        }
    }
    check_components! { TestApp { UserCreatorComponent, UserGetterComponent, UserUpdaterComponent, PostCreatorComponent, PostGetterComponent, PostUpdaterComponent, PostDeleterComponent, UsernameCensorComponent, SpamMessageDetectorComponent, } }
}

mod equivalent_namespace_body_binding {
    use super::domain::*;
    use super::give_component_groups_a_path::*;
    use cgp::prelude::*;
    cgp_namespace! {
        new DefaultAppComponents: DefaultNamespace
    }
    cgp_namespace! {
        DefaultAppComponents {
            @app.core.user.UserGetterComponent: GetUserWithPostgres,
        }
    }
    #[derive(HasField)]
    pub struct App {
        pub database: PostgresDb,
    }
    delegate_components! { App { namespace DefaultAppComponents; } }
    check_components! { App { UserGetterComponent, } }
}

mod replace_presets_with_namespace_configuration {
    use super::domain::*;
    use super::give_component_groups_a_path::*;
    use cgp::prelude::*;
    cgp_namespace! {
        new PostgresAppNamespace: DefaultNamespace {
            @app.core.user.UserCreatorComponent:
                FilterCensoredUsername<CreateUserWithPostgres>,
            @app.core.user.UserGetterComponent: GetUserWithPostgres,
            @app.core.user.UserUpdaterComponent: UpdateUserWithPostgres,
        }
    }
    #[derive(HasField)]
    pub struct ProductionApp {
        pub database: PostgresDb,
    }

    delegate_components! {
        ProductionApp {
            namespace PostgresAppNamespace;

            @app.extra.content_filter.UsernameCensorComponent: AiUserCensor,
        }
    }

    check_components! {
        ProductionApp {
            UserCreatorComponent,
            UserGetterComponent,
            UserUpdaterComponent,
            UsernameCensorComponent,
        }
    }
}

mod update_attribute_syntax {
    use cgp::prelude::*;
    #[cgp_component(Loader)]
    #[use_type(HasErrorType.Error)]
    pub trait CanLoad {
        fn load(&self) -> Result<(), Error>;
    }
    #[cgp_impl(new Load)]
    #[use_type(HasErrorType.{Error = AppError})]
    impl Loader {
        fn load(&self) -> Result<(), Error> {
            Ok(())
        }
    }
    #[derive(Debug)]
    pub struct AppError;
    pub struct App;
    impl HasErrorType for App {
        type Error = AppError;
    }
    delegate_and_check_components! { App { LoaderComponent: Load } }
}
