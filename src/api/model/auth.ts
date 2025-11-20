export interface GetCaptchaResp {
  image: string;
  info: string;
}

export interface UserLoginReq {
  email: string;
  password: string;
  captcha: string;
  info: string;
}

export interface UserLoginResp {
  token: string;
}
