export interface ReqLogin {
  email: string;
  password: string;
}

export interface ResLogin {
  token: string;
}

export interface ResUser {
  id: string;
  name: string;
  has_avatar: boolean;
  avatar: string;
}
