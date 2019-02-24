use sapper::{
    Request, 
    Response, 
    Result as SapperResult, 
    Error as SapperError, 
    Module as SapperModule,
    Router as SapperRouter};
use sapper_std::{JsonParams, SessionVal, render};

use crate::serde_json;

use crate::db;
// introduce macros
use crate::AppWebContext;

use dataservice::user::{
    UserLogin,
    UserSignUp
};


pub struct UserPage;

impl UserPage {

    pub fn page_login_with3rd(req: &mut Request) -> SapperResult<Response> {
        let mut web = req_ext_entity!(AppWebContext).unwrap();

        res_html!("forum/login_with3rd.html", web)
    }

    pub fn page_login_with_admin(req: &mut Request) -> SapperResult<Response> {
        let mut web = req_ext_entity!(AppWebContext).unwrap();

        res_html!("forum/login_with_admin.html", web)
    }

    pub fn user_register(req: &mut Request) -> SapperResult<Response> {

        let params = get_form_params!(req);
        let account = t_param!(params, "account");
        let password = t_param!(params, "password");
        let nickname = t_param!(params, "nickname");

        let user_signup = UserSignUp {
            account,
            password,
            nickname
        };

        // TODO: need to check the result of this call
        let _ = user_signup.sign_up(None);

        // redirect to login with account and password
        res_redirect!("/login_with_admin")
    }

    pub fn user_login(req: &mut Request) -> SapperResult<Response> {

        let params = get_form_params!(req);
        let account = t_param!(params, "account");
        let password = t_param!(params, "password");

        let user_login = UserLogin {
            account,
            password
        };

        // use dataservice logic
        let cookie = user_login.verify_login().unwrap();

        let mut response = Response::new();
        let _ = set_cookie(
            &mut response,
            "rusoda_session".to_string(),
            cookie,
            None,
            Some("/".to_string()),
            None,
            Some(60*24*3600),
        );

        // redirect to index
        set_response_redirect!("/");

        Ok(response)
    }

    pub fn user_login_with_github(req: &mut Request) -> SapperResult<Response> {

        let params = get_form_params!(req);
        let code = t_param!(params, "code");


        let token_r = get_github_token(&code);
        if token_r.is_err() {
            return res_400!("get github token code err");
        }
        let access_token = token_r.unwrap();
        let github_user_info: GithubUserInfo = get_github_user_info(&access_token).unwrap();

        let account = github_user_info.account;
        let password;

        match Ruser::get_by_account(&account) {
            Some(user) => {
                // already exists
                password = user.password;
            },
            None => {
                password = random_string(8);
                // register it
                let user_signup = UserSignUp {
                    account,
                    password,
                    nickname: account,
                };
                // TODO: check the result
                let _ = user_signup.sign_up(Some(github_user_info.github_address));
            }
        }

        // next step auto login
        let user_login = UserLogin {
            account,
            password
        };

        // use dataservice logic
        let cookie = user_login.verify_login().unwrap();

        let mut response = Response::new();
        let _ = set_cookie(
            &mut response,
            "rusoda_session".to_string(),
            cookie,
            None,
            Some("/".to_string()),
            None,
            Some(60*24*3600),
        );

        // redirect to index
        set_response_redirect!("/");

        Ok(response)
    }
}


impl SapperModule for UserPage {
    fn before(&self, req: &mut Request) -> SapperResult<()> {

        Ok(())
    }

    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/login_with3rd", Self::page_login_with3rd);
        router.get("/login_with_admin", Self::page_login_with_admin);

        router.post("/s/user/register", Self::user_register);
        router.post("/s/user/login", Self::user_login);
        // this url will be called by remote github oauth2 server
        router.post("/s/user/login_with_github", Self::user_login_with_github);

        Ok(())
    }
}


