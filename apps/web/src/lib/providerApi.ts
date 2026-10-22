import {BACKEND_MODE,request} from './api';
import type {TextProvider,ProviderInput,TextModel,TextModelInput,PublicTextModel,ProviderTest} from '../features/providers/types';
function requireServer(){if(!BACKEND_MODE)throw Error('请启动 Rust HTTP 模式后管理供应商；演示模式不会保存密钥。');}
export async function listProviders(){requireServer();return request<TextProvider[]>('/admin/providers');}
export async function saveProvider(input:ProviderInput,id?:string){requireServer();return request<TextProvider>(`/admin/providers${id?`/${id}`:''}`,{method:id?'PUT':'POST',body:JSON.stringify(input)});}
export async function testProvider(id:string,revision:number){requireServer();return request<ProviderTest>(`/admin/providers/${id}/test`,{method:'POST',body:JSON.stringify({revision})});}
export async function listTextModels(){requireServer();return request<TextModel[]>('/admin/text-models');}
export async function saveTextModel(input:TextModelInput,id?:string){requireServer();return request<TextModel>(`/admin/text-models${id?`/${id}`:''}`,{method:id?'PUT':'POST',body:JSON.stringify(input)});}
export async function listTextModelCatalog(){return BACKEND_MODE?request<PublicTextModel[]>('/text-model-catalog'):[];}