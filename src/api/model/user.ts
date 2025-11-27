export interface ReqSync {
  userId: string;
  email: string;
  password: string;
}

export interface ResSync {
  token: string;
}

export interface ResUser {
  id: string;
  name: string;
  hasAvatar: boolean;
  avatar: string;
}
