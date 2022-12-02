
export interface DeveloperProps {
    name: string;
    email: string;
}

export interface ProjectStoreProps {
    info: ProjectProps | null;
    config: GitConfigProps | null;
    developer: DeveloperProps[];
    current: string,
    local_branche: GitBranchItemProps[],
    remote_branche: GitBranchItemProps[],
    statusList: StatusItem[];
}

// 配置信息
export interface GitConfigProps {
    user_name: string;
    user_email: string;
    remote_url: string;
}

export interface GitBranchItemProps {
    name: string;
    is_local: boolean;
}

export interface GitBranchInfoProps {
    current: string;
    local_list: GitBranchItemProps[];
    remote_list: GitBranchItemProps[];
}

export interface GitChangeLinesParamsProps {
    branch: string;
    compare: string;
    start: string;
    end: string;
    excludes: string[];
    author: string;
}

export interface GitChangeLinesProps {
    add_lines: number;
    subtract_lines: number;
    total_lines: number;
}