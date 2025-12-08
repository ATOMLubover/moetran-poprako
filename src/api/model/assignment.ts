// Assignment (派活) 相关数据模型

export interface ResAssignment {
  projId: string;
  projName: string;
  projsetSerial: number;
  projsetIndex: number;
  memberId: string;
  username: string;
  isTranslator: boolean;
  isProofreader: boolean;
  isTypesetter: boolean;
  isPrincipal?: boolean;
  isRedrawer: boolean;
  updatedAt: number; // Unix timestamp (seconds)
}
