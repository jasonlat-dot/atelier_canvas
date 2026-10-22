export type TextProvider={id:string;name:string;protocol:'chat_completions';baseUrl:string;enabled:boolean;requireApiKey:boolean;hasApiKey:boolean;revision:number;lastTestStatus:'ok'|'error'|null;lastTestCode:string|null;lastTestAt:string|null};
export type ProviderInput={name:string;protocol:'chat_completions';baseUrl:string;enabled:boolean;requireApiKey:boolean;revision:number;apiKey?:string;clearApiKey?:boolean};
export type TextModel={id:string;providerId:string;modelId:string;label:string;enabled:boolean;contextWindow:number;maxOutputTokens:number;revision:number};
export type TextModelInput=Omit<TextModel,'id'>;
export type PublicTextModel=Pick<TextModel,'id'|'label'|'contextWindow'|'maxOutputTokens'>;
export type ProviderTest={ok:boolean;code:string;models:string[];latencyMs:number};