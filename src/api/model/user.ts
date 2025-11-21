interface UserProfile {
  poprakoId: string;
  poprakoToken: string;
  username: string;
  email: string;
  moetranId: string;
  moetranToken: string;
}

interface MoeUserLoginReq {
  email: string;
  password: string;
  captchaToken: string;
}

interface MoeUserLoginRes {
  token: string;
}
