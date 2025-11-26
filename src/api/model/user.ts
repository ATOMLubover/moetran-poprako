export interface ReqSync {
  user_id: string;
  email: string;
  password: string;
}

export interface ResSync {
  token: string;
}

export interface ResUser {
  id: string;
  name: string;
  has_avatar: boolean;
  avatar: string;
}
