export interface ResMember {
  // PopRaKo 返回的对应用户 id，用于与本地用户 id 比对
  userId: string;
  memberId: string;
  username: string;
  isAdmin: boolean;
  isTranslator: boolean;
  isProofreader: boolean;
  isTypesetter: boolean;
  isPrincipal: boolean;
}

export interface ResMemberBrief {
  memberId: string;
  username: string;
}

export interface ResMemberInfo {
  memberId: string;
  isAdmin: boolean;
  isTranslator: boolean;
  isProofreader: boolean;
  isTypesetter: boolean;
  isPrincipal: boolean;
}
