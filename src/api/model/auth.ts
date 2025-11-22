export interface ResCaptcha {
  image: string;
  info: string;
}

export interface ReqToken {
  email: string;
  password: string;
  captcha: string;
  info: string;
}

export interface ResToken {
  token: string;
}
