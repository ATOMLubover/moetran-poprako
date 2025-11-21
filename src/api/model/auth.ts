export interface ResCaptcha {
  image: string;
  info: string;
}

export interface ReqMoeToken {
  email: string;
  password: string;
  captcha: string;
  info: string;
}

export interface ResMoeToken {
  token: string;
}
