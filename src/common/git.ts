import { GitBranchInfoProps, GitConfigProps, GitChangeLinesParamsProps, GitChangeLinesProps } from '../types';
import { invoke } from "@tauri-apps/api/tauri";
import logger from "./logger";

/**
 * 获取项目配置信息
 * @param path 项目地址
 */
export const getConfigInfo = async (path: string) => {
    if (!path) return null;
    const config: GitConfigProps = await invoke('get_config', { path });
    logger.success('获取config为 ->', config);
    return config;
}

/**
 * 获取开发人员
 * @param path 项目地址
 */
export const getDeveloper = async (path: string) => {
    // 获取项目开发人员
    const authors: string[] = await invoke('get_developer', { path });
    logger.success('获取的开发者为 ->', authors);
    return authors.map(item => {
        const arr = item.trim().replaceAll('>', '').split('<');
        return {
            name: arr[0],
            email: arr[1],
        };
    });
}

/**
 * 获取分支列表
 * @param path 项目地址
 */
export const getBranchInfo = async (path: string) => {
    const info: GitBranchInfoProps = await invoke('get_branche_info', { path });
    logger.success('获取的分支信息为 ->', info);
    return info;
}

/**
 * 获取分支创建时间
 * @param path 项目地址
 * @param branch 本地分支
 */
export const getBranchCreateDate = async (path: string, branch: string) => {
    if (!path || !branch) return '';
    // 获取分支创建时间
    const dateInfo: string = await invoke('get_branch_create_date', {
        path,
        branch,
    });
    logger.success('获取的分支创建信息为 ->', dateInfo);
    if (!dateInfo) return '';
    const list: any = dateInfo.match(/\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}/g);
    return list[0];
}

/**
 * 计算变动行数
 */
export const calcChangeLines = async (path: string, params: GitChangeLinesParamsProps) => {
    if (!path) {
        return '';
    }
    const info: string = await invoke('calc_change_lines', { path, params });
    const arr = info.split(',');
    let obj: any = {};
    for (let i = 0; i < arr.length; i++) {
        const items = arr[i].split(':');
        obj = {
            ...obj,
            [items[0]]: items[1] || 0,
        };
    }
    return obj;
};