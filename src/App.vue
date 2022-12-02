<template>
    <div class="v-app">
        <Form ref="calcForm" class="margin-top-md" :label-width="100">
            <FormItem label="项目地址">
                <Input v-model="projectPath" style="width: 350px" placeholder="项目地址">
                <template #append>
                    <Button @click="handleSelectFolder">选择地址</Button>
                </template>
                </Input>
            </FormItem>
        </Form>
        <Form ref="calcForm" class="margin-top-md" :model="formItem" :rules="ruleValidate" :label-width="100">
            <FormItem label="分支" prop="branch">
                <Select v-model="formItem.branch" :disabled="isDisable" style="width: 350px" clearable filterable
                    @on-change="handleBranchChange">
                    <Option v-for="item in local_branche" :value="item.name" :key="item.name">{{ item.name }}</Option>
                </Select>
            </FormItem>
            <FormItem label="开始时间" prop="start">
                <DatePicker v-model="formItem.start" :disabled="isDisable" type="date" placeholder="选择开始时间"
                    style="width: 350px" />
            </FormItem>
            <FormItem label="结束时间" prop="end">
                <DatePicker v-model="formItem.end" :disabled="isDisable" type="date" placeholder="选择结束时间"
                    style="width: 350px" />
            </FormItem>
            <FormItem label="开发者" prop="author">
                <Select v-model="formItem.author" :disabled="isDisable" style="width: 350px" clearable filterable>
                    <Option v-for="item in developer" :value="`${item.name} <${item.email}>`" :key="item.email">{{
                    item.name }}</Option>
                </Select>
            </FormItem>
            <FormItem label="剔除文件" prop="excludes">
                <Tag v-for="item in formItem.excludes" :key="item" :name="item" closable
                    @onClose="event => handleDeleteExcludeItem(event, item)">{{ item }}</Tag>
                <div class="flex-row">
                    <Button icon="ios-add" :disabled="isDisable" type="dashed" size="small"
                        @click="handleAddExcludeFileItem">添加文件</Button>
                    <Button icon="ios-add" class="margin-left" :disabled="isDisable" type="dashed" size="small"
                        @click="handleAddExcludeFolderItem">添加目录</Button>
                </div>
            </FormItem>
            <FormItem>
                <Button type="primary" :disabled="isDisable" :loading="loading" @click="calcAction">统计</Button>
            </FormItem>
            <Spin size="large" fix :show="isLoading"></Spin>
        </Form>
        <Modal v-model="showCalcResult" title="代码变动统计" @on-cancel="() => showCalcResult = false" footer-hide
            :mask-closable="false">
            <div class="v-calc-result flex-column-center" v-if="result">
                <div class="result-value">{{ result }}</div>
                <div class="flex-row-center margin-top-md">
                    <Button type="primary" :loading="loading" @click="handleCopyAction">复制</Button>
                    <Button class="margin-left" @click="() => showCalcResult = false">关闭</Button>
                </div>
            </div>
        </Modal>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { Form, FormItem, Input, Button, Select, Option, Tag, DatePicker, Spin, Modal } from 'view-ui-plus';
import dayjs from 'dayjs';
import { open } from '@tauri-apps/api/dialog';
import { homeDir } from '@tauri-apps/api/path';
import { invoke } from '@tauri-apps/api/tauri';
import { DeveloperProps, GitBranchItemProps, GitChangeLinesParamsProps, GitChangeLinesProps } from './types';
import { calcChangeLines, getBranchCreateDate, getBranchInfo, getDeveloper } from './common/git';
import { writeText } from '@tauri-apps/api/clipboard';
import logger from './common/logger';
import { errorTip, successTip } from './common/tip';

const DATE_FORMAT = 'YYYY-MM-DD';
const loading = ref<boolean>(false);
const calcForm = ref();
const current = ref<string>('');
const local_branche = ref<GitBranchItemProps[]>([]);
const developer = ref<DeveloperProps[]>();
const result = ref<string>('');
const projectPath = ref<string>('');
const isDisable = ref<boolean>(true);
const isLoading = ref<boolean>(false);
const showCalcResult = ref<boolean>(false);
const formItem = ref<any>({
    branch: '',
    compare: '',
    start: '',
    end: '',
    author: '',
    excludes: [],
});

watch(projectPath, async (newPath, oldPath) => {
    if (!newPath) {
        isDisable.value = true;
        return;
    }
    isLoading.value = true;
    // 获取项目分支
    const branchInfo = await getBranchInfo(newPath);
    local_branche.value = branchInfo.local_list;
    current.value = branchInfo.current;
    formItem.value.branch = current.value;
    const dateInfo = await getBranchCreateDate(newPath, current.value)
    formItem.value.start = dayjs(dateInfo).format(DATE_FORMAT);
    formItem.value.end = dayjs().format(DATE_FORMAT);
    // 获取开发者
    const developerInfo = await getDeveloper(newPath);
    developer.value = developerInfo;
    isDisable.value = false;
    isLoading.value = false;
});


const validatorPath = async (rule: any, value: any, callback: any) => {
    if (value === '') {
        callback(new Error('项目地址不能为空'));
    }
    // 判断是否是git项目
    const isGitProject = await invoke('is_git', { path: value });
    if (!isGitProject) {
        callback(new Error('选择的地址不是 Git 项目'));
    }
    callback();
}

const ruleValidate = {
    branch: [
        { required: true, message: '分支不能为空', trigger: 'blur' }
    ],
    path: [
        { required: true, validator: validatorPath, trigger: 'change' },
    ],
};

// 选择项目地址
const handleSelectFolder = async () => {
    const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: await homeDir(),
    });
    const folder = selected as string;
    // 判断是否是git项目
    const isGitProject = await invoke('is_git', { path: folder });
    if (!isGitProject) {
        errorTip('不是Git项目');
        return;
    }
    projectPath.value = folder;
}

const handleBranchChange = async (value: string) => {
    const dateInfo = await getBranchCreateDate(projectPath.value || '', value)
    formItem.value.start = dayjs(dateInfo).format(DATE_FORMAT);
}

const handleAddExcludeFileItem = async () => {
    const selected = await open({
        title: '选择剔除文件',
        defaultPath: projectPath.value,
        directory: false,
        multiple: false,
    });
    if (!selected) return;
    const base = formItem.value.excludes;
    const target = selected as string;
    base.push(target.replace(`${projectPath.value}/`, ''));
    formItem.value.excludes = base;
}

const handleAddExcludeFolderItem = async () => {
    const selected = await open({
        title: '选择剔除目录',
        defaultPath: projectPath.value,
        directory: true,
        multiple: false,
    });
    if (!selected) return;
    const base = formItem.value.excludes;
    const target = selected as string;
    base.push(target.replace(`${projectPath.value}/`, ''));
    formItem.value.excludes = base;
}

const handleDeleteExcludeItem = (event: any, name: string) => {
    console.log(event, name);
    const base = formItem.value.excludes.filter((item: string) => item !== name);
    formItem.value.excludes = base;
}

const calcAction = async () => {
    calcForm.value.validate(async (flag: boolean) => {
        if (!flag) return false;
        loading.value = true;
        const params: GitChangeLinesParamsProps = {
            branch: formItem.value.branch || '',
            compare: formItem.value.compare || '',
            start: dayjs(formItem.value.start).subtract(1, 'd').format(DATE_FORMAT),
            end: dayjs(formItem.value.end).add(1, 'd').format(DATE_FORMAT),
            author: formItem.value.author || '',
            excludes: formItem.value.excludes || [],
        };
        logger.info('计算参数为 ->', params);
        const targetValue: GitChangeLinesProps = await calcChangeLines(projectPath.value || '', params);
        result.value = `增加：${targetValue.add_lines}行、删除：${targetValue.subtract_lines}行、总共：${targetValue.total_lines}行`;
        logger.info('计算结果为 ->', result.value);
        loading.value = false;
        showCalcResult.value = true;
    });
}

const handleCopyAction = async () => {
    await writeText(result.value);
    successTip('复制成功');
    showCalcResult.value = false;
}

</script>

<style scoped>

</style>