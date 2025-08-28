import axios from 'axios'

// 创建axios实例
const api = axios.create({
    baseURL: '/api', // 使用相对路径，假设前端和后端在同一域名下
    timeout: 10000,
    headers: {
        'Content-Type': 'application/json',
    },
})

// 请求拦截器
api.interceptors.request.use(
    (config) => {
        // 在发送请求之前做些什么
        const token = localStorage.getItem('token')
        if (token) {
            config.headers.Authorization = `Bearer ${token}`
        }
        return config
    },
    (error) => {
        // 对请求错误做些什么
        return Promise.reject(error)
    },
)

// 响应拦截器
api.interceptors.response.use(
    (response) => {
        // 对响应数据做些什么
        return response
    },
    (error) => {
        // 对响应错误做些什么
        if (error.response?.status === 401) {
            // 如果是未授权错误，清除token并重定向到登录页面
            localStorage.removeItem('token')
            window.location.href = '/login'
        }
        return Promise.reject(error)
    },
)

// 登录API
export const login = (username, password) => {
    return api.post('/login', { username, password })
}

// 获取漏洞列表
export const getVulnerabilities = (params) => {
    return api.get('/vulns', {
        params: {
            page_no: params.pageNo,
            page_size: params.pageSize,
            cve: params.cve,
            title: params.title,
            pushed: params.pushed,
            source_name: params.source_name,
        },
    })
}

// 获取漏洞详情
export const getVulnerabilityDetail = (id) => {
    return api.get(`/vulns/${id}`)
}

// 获取同步任务
export const getSyncDataTask = () => {
    return api.get('/sync_data_task')
}

// 创建或更新同步任务
export const createOrUpdateSyncDataTask = (data) => {
    return api.post('/sync_data_task', data)
}

// 获取插件列表
export const getPlugins = () => {
    return api.get('/plugins')
}

// 获取钉钉机器人配置
export const getDingBotConfig = () => {
    return api.get('/ding_bot_config')
}

// 创建或更新钉钉机器人配置
export const createOrUpdateDingBotConfig = (data) => {
    return api.post('/ding_bot_config', data)
}

export default api
