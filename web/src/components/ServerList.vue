<template>
  <div class="server-management">
    <!-- 消息模态框 -->
    <MessageModal
      v-if="showMessageModal"
      :title="modalTitle"
      :message="modalMessage"
      @close="showMessageModal = false"
    />
    
    <!-- 全局配置区域 -->
    <div class="global-config-section">
      <h3>全局配置</h3>
      <div class="config-card">
        <div v-for="(item, index) in configData.global" :key="index" class="config-item">
          <strong>{{ item[0] }}:</strong> {{ item[1] }}
          <button @click="editGlobalConfig(index)" class="save-button">编辑</button>
          <button @click="removeGlobalConfig(index)" class="delete-button">删除</button>
        </div>
      </div>
      <button @click="showAddGlobalConfig = true" class="add-button">
        添加全局配置
      </button>
    </div>

    <!-- 主机列表区域 -->
    <div class="server-list-section">

      <div class="section-header">
        <h3>服务器列表</h3>
        <div>
          <button @click="showAddServerForm = true" class="add-button">
            添加新主机
          </button>
          <!--button @click="saveConfig" class="save-button">保存配置</button-->
        </div>
      </div>

      <!-- 过滤区域 -->
      <div class="filter-section">
        <div class="filter-control">
          <label for="group-filter">按组过滤:</label>
          <select id="group-filter" v-model="filterGroup" @change="() => loadConfig()">
            <option value="">全部</option>
            <option v-for="group in availableGroups" :key="group" :value="group">
              {{ group }}
            </option>
          </select>
        </div>
        <div class="filter-control">
          <label for="tag-filter">按标签过滤:</label>
          <select id="tag-filter" v-model="filterTag" @change="() => loadConfig()">
            <option value="">全部</option>
            <option v-for="tag in availableTags" :key="tag" :value="tag">
              {{ tag }}
            </option>
          </select>
        </div>
        <button @click="resetFilters" class="reset-button">重置过滤</button>
      </div>

      
      
      <!-- 桌面视图：表格布局 -->
      <div class="desktop-view">
        <table class="server-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>主机标签</th>
              <th>连接信息</th>
              <th>组</th>
              <th>标签</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(server, index) in configData.servers" :key="index">
              <td>{{ index }}</td>
              <td>{{ server.host_tag }}</td>
              <td>{{ server.user }}@{{ server.hostname }}:{{ server.port }}</td>
              <td>{{ server.group || 'N/A' }}</td>
              <td>
                <span v-for="(tag, index) in server.tags" :key="index" class="tag">
                  {{ tag }}
                </span>
              </td>
              <td>
                <button @click="connect(server)" class="add-button">连接</button>
                <button @click="edit(server, index)" class="save-button">编辑</button>
                <button @click="confirmDelete(index)" class="delete-button">删除</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      
      <!-- 移动视图：卡片布局 -->
      <div class="mobile-view">
        <div v-for="(server, index) in configData.servers" :key="index" class="server-card">
          <div class="card-header">
            <div class="server-id">ID: {{ index }}</div>
            <div class="server-tag">{{ server.host_tag }}</div>
          </div>
          
          <div class="card-body">
            <div class="info-row">
              <span class="label">连接信息:</span>
              <span>{{ server.user }}@{{ server.hostname }}:{{ server.port }}</span>
            </div>
            <div class="info-row">
              <span class="label">组:</span>
              <span>{{ server.group || 'N/A' }}</span>
            </div>
            <div class="info-row">
              <span class="label">标签:</span>
              <span>
                <span v-for="(tag, tagIndex) in server.tags" :key="tagIndex" class="tag">
                  {{ tag }}
                </span>
              </span>
            </div>
            <div class="info-row" v-if="server.forward_agent || server.dynamic_forward || server.local_forward || server.proxy_jump">
              <span class="label">转发:</span>
              <span>
                <span v-if="server.forward_agent" class="tag">Agent</span>
                <span v-if="server.dynamic_forward" class="tag">Dyn:{{ server.dynamic_forward }}</span>
                <span v-if="server.local_forward" class="tag">
                  Loc:{{ typeof server.local_forward === 'string' ? server.local_forward :
                    `${server.local_forward.local_port}:${server.local_forward.remote_host}:${server.local_forward.remote_port}` }}
                </span>
                <span v-if="server.proxy_jump" class="tag">Jump:{{ server.proxy_jump }}</span>
              </span>
            </div>
          </div>
          
          <div class="card-actions">
            <button @click="connect(server)" class="add-button">连接</button>
            <button @click="edit(server)" class="save-button">编辑</button>
            <button @click="confirmDelete(index)" class="delete-button">删除</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加新主机表单 -->
    <div v-if="showAddServerForm" class="modal">
      <div class="modal-content">
        <span class="close" @click="showAddServerForm = false">&times;</span>
        <h3>添加新主机配置</h3>
        <form @submit.prevent="addNewServer">
          <div class="form-group">
            <label>主机标签:</label>
            <input v-model="newServer.host_tag" required />
          </div>
          <div class="form-group">
            <label>主机地址:</label>
            <input v-model="newServer.hostname" required />
          </div>
          <div class="form-group">
            <label>用户名:</label>
            <input v-model="newServer.user" required />
          </div>
          <div class="form-group">
            <label>端口:</label>
            <input v-model="newServer.port" type="number" />
          </div>
          <div class="form-group">
            <label>分组:</label>
            <input v-model="newServer.group" />
          </div>
          <div class="form-group">
            <label>标签 (逗号分隔):</label>
            <input v-model="newServer.tagsInput" />
          </div>
          <div class="form-group">
            <label>
              Forward Agent
            </label>
            <input type="checkbox" v-model="newServer.forwardAgent">
          </div>
          <div class="form-group">
            <label>Dynamic Forward</label>
            <input v-model="newServer.dynamicForward" placeholder="e.g. 8080">
          </div>
          <div class="form-group">
            <div class="form-group">
              <label>Local Forward</label>
              <div class="local-forward-fields">
                <input v-model="newServer.localForward.local_port"
                       type="number"
                       placeholder="本地端口"
                       class="port-input">
                <input v-model="newServer.localForward.remote_host"
                       placeholder="远程主机"
                       class="host-input">
                <span>:</span>
                <input v-model="newServer.localForward.remote_port"
                       type="number"
                       placeholder="远程端口"
                       class="port-input">
              </div>
            </div>
          </div>
          <div class="form-group">
            <label>Proxy Jump</label>
            <input v-model="newServer.proxyJump" placeholder="e.g. jump-host">
          </div>
          <button type="submit">保存</button>
        </form>
      </div>
    </div>

    <!-- 添加全局配置表单 -->
    <div v-if="showAddGlobalConfig" class="modal">
      <div class="modal-content">
        <span class="close" @click="showAddGlobalConfig = false">&times;</span>
        <h3>添加全局配置</h3>
        <form @submit.prevent="addGlobalConfig">
          <div class="form-group">
            <label>配置项:</label>
            <input v-model="newConfig.key" placeholder="例如: ServerAliveInterval" required />
          </div>
          <div class="form-group">
            <label>值:</label>
            <input v-model="newConfig.value" placeholder="例如: 30" required />
          </div>
          <div class="form-actions">
            <button @click="showAddGlobalConfig = false" class="delete-button">取消</button>
            <button type="submit" class="add-button">添加</button>
          </div>
        </form>
      </div>
    </div>

    <!-- 编辑全局配置表单 -->
    <div v-if="showEditGlobalConfig" class="modal" style="z-index: 1001;">
      <div class="modal-content">
        <span class="close" @click="showEditGlobalConfig = false">&times;</span>
        <h3>编辑全局配置</h3>
        <form @submit.prevent="saveEditedGlobalConfig">
          <div class="form-group">
            <label>配置项</label>
            <input v-model="editingGlobalConfig.data.key" required>
          </div>
          <div class="form-group">
            <label>值</label>
            <input v-model="editingGlobalConfig.data.value" required>
          </div>
          <div class="form-actions">
            <button @click="showEditGlobalConfig = false" class="delete-button">取消</button>
            <button class="add-button">保存</button>
          </div>
        </form>
      </div>
    </div>

    <!-- 消息提示模态框 -->
    <div v-if="showMessageModal" class="modal">
      <div class="modal-content message-modal">
        <div class="message-header">
          <h4>{{ messageModal.title }}</h4>
        </div>
        <div class="message-body">
          <p>{{ messageModal.message }}</p>
        </div>
        <div class="message-footer">
          <button @click="handleMessageConfirm" class="confirm-button">确认</button>
        </div>
      </div>
    </div>
    
    <!-- 编辑服务器表单 -->
    <div v-if="showEditServerForm" class="modal" style="z-index: 1001;">
      <div class="modal-content">
        <span class="close" @click="showEditServerForm = false">&times;</span>
        <h3>编辑服务器</h3>
        <form @submit.prevent="saveEditedServer">
          <div class="form-group">
            <label>主机标识</label>
            <input v-model="editingServer.data.host_tag" required>
          </div>
          <div class="form-group">
            <label>主机名/IP</label>
            <input v-model="editingServer.data.hostname" required>
          </div>
          <div class="form-group">
            <label>用户名</label>
            <input v-model="editingServer.data.user" required>
          </div>
          <div class="form-group">
            <label>端口</label>
            <input v-model="editingServer.data.port" type="number">
          </div>
          <div class="form-group">
            <label>分组</label>
            <input v-model="editingServer.data.group">
          </div>
          <div class="form-group">
            <label>标签(逗号分隔)</label>
            <input v-model="editingServer.data.tagsInput">
          </div>
          <div class="form-group">
            <label>Forward Agent</label>
            <input type="checkbox" v-model="editingServer.data.forwardAgent">
          </div>
          <div class="form-group">
            <label>Dynamic Forward</label>
            <input v-model="editingServer.data.dynamicForward" placeholder="e.g. 8080">
          </div>
          <div class="form-group">
            <div class="form-group">
              <label>Local Forward</label>
              <div class="local-forward-fields">
                <input v-model="editingServer.data.localForward.local_port"
                       type="number"
                       placeholder="本地端口"
                       class="port-input">
                <input v-model="editingServer.data.localForward.remote_host"
                       placeholder="远程主机"
                       class="host-input">
                <span>:</span>
                <input v-model="editingServer.data.localForward.remote_port"
                       type="number"
                       placeholder="远程端口"
                       class="port-input">
              </div>
            </div>
          </div>
          <div class="form-group">
            <label>Proxy Jump</label>
            <input v-model="editingServer.data.proxyJump" placeholder="e.g. jump-host">
          </div>
          <div class="form-actions">
            <button @click="showEditServerForm = false" class="delete-button">取消</button>
            <button class="add-button">保存</button>
          </div>
        </form>
      </div>
    </div>

  </div>

  
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import * as tauriapi from '@tauri-apps/api';
import { show } from '@tauri-apps/api/app';

// 配置数据
const configData = ref({
  global: [], // 存储格式: [{key: "", value: ""}]
  servers: []
})

// 过滤状态
const filterGroup = ref('')
const filterTag = ref('')

// 检查是否处于过滤状态
const isFiltered = computed(() => {
  return filterGroup.value || filterTag.value
})

// 计算可用分组和标签
const availableGroups = computed(() => {
  const groups = new Set()
  configData.value.servers.forEach(server => {
    if (server.group) groups.add(server.group)
  })
  return Array.from(groups).sort()
})

const availableTags = computed(() => {
  const tags = new Set()
  configData.value.servers.forEach(server => {
    if (server.tags) {
      server.tags.forEach(tag => tags.add(tag))
    }
  })
  return Array.from(tags).sort()
})

// 重置过滤器
function resetFilters() {
  filterGroup.value = ''
  filterTag.value = ''
  loadConfig()
}

// 新服务器表单状态
const showAddServerForm = ref(false)
const newServer = ref({
  host_tag: '',
  hostname: '',
  user: '',
  port: 22,
  group: '',
  tagsInput: '',
  forwardAgent: false,
  dynamicForward: '',
  localForward: {
    local_port: '',
    remote_host: '',
    remote_port: ''
  },
  proxyJump: ''
})

// 新全局配置表单状态
const showAddGlobalConfig = ref(false)
const showEditGlobalConfig = ref(false)
const newConfig = ref({
  key: '',
  value: ''
})
const editingGlobalConfig = ref({
  index: -1,
  data: {
    key: '',
    value: ''
  }
})

// 消息模态框状态
const showMessageModal = ref(false)
const messageModal = ref({
  title: '',
  message: ''
})

// 删除确认状态
const pendingDeleteIndex = ref(-1)

// 显示消息模态框
function showMessage(title, message) {
  messageModal.value = { title, message }
  showMessageModal.value = true
}

// 显示删除确认对话框
function confirmDelete(index) {
  if (isFiltered.value) {
    showMessage('操作禁止', '过滤状态下不能删除服务器')
    return
  }
  const server = configData.value.servers[index]
  pendingDeleteIndex.value = index
  showMessage('确认删除', `确定要删除服务器 ${server.host_tag} (${server.user}@${server.hostname}) 吗?`)
}

// 执行删除操作
async function removeServer(index) {
  if (isFiltered.value) {
    showMessage('操作禁止', '过滤状态下不能删除服务器')
    return
  }
  try {
    configData.value.servers.splice(index, 1)
    await saveConfig()
    showMessage('成功', '服务器已删除')
  } catch (error) {
    showMessage('错误', `删除服务器失败: ${error}`)
  }
}

// 关闭消息模态框
function closeMessageModal() {
  showMessageModal.value = false
  pendingDeleteIndex.value = -1
}

// 加载配置
async function loadConfig() {
  try {
    const params = {
      group: filterGroup.value || null,
      tags: filterTag.value || null
    }
    
    console.log('Sending filter params:', params) // 调试参数
    const response = await tauriapi.core.invoke('get_servers', params)
    console.log('Received filtered data:', response) // 调试响应
    
    // 确保响应式更新
    configData.value = {
      global: response.global,
      servers: response.servers
    }
  } catch (error) {
    showMessage('错误', `加载配置失败: ${error}`)
  }
}

// 保存配置
async function saveConfig() {
  try {
    const configToSave = {
      global: configData.value.global,
      servers: configData.value.servers
    }
    await tauriapi.core.invoke('save_servers', { config: configToSave })
    showMessage('成功', '配置保存成功!')
  } catch (error) {
    console.error('保存配置失败:', error)
    showMessage('错误', '保存配置失败: ' + error)
  }
}

// 处理消息确认
function handleMessageConfirm() {
  if (pendingDeleteIndex.value !== -1) {
    removeServer(pendingDeleteIndex.value)
  }
  closeMessageModal()
}

// 添加全局配置
function addGlobalConfig() {
  if (!newConfig.value.key || !newConfig.value.value) {
    showMessage('错误', '配置项和值不能为空')
    return
  }
  
  configData.value.global.push([
    newConfig.value.key,
    newConfig.value.value
  ])
  
  newConfig.value = { key: '', value: '' }
  showAddGlobalConfig.value = false
  saveConfig()
}

// 编辑全局配置
function editGlobalConfig(index) {
  editingGlobalConfig.value = {
    index,
    data: {
      key: configData.value.global[index][0],
      value: configData.value.global[index][1]
    }
  }
  showEditGlobalConfig.value = true
}

// 保存编辑后的全局配置
async function saveEditedGlobalConfig() {
  if (editingGlobalConfig.value.index >= 0) {
    configData.value.global[editingGlobalConfig.value.index] = [
      editingGlobalConfig.value.data.key,
      editingGlobalConfig.value.data.value
    ]
    showEditGlobalConfig.value = false
    await saveConfig()
  }
}

// 删除全局配置
async function removeGlobalConfig(index) {
  configData.value.global.splice(index, 1)
  await saveConfig()
}

// 连接服务器
async function connect(server) {
  try {
    console.log('连接到服务器:', server.host_tag)
    await tauriapi.core.invoke('connect_to_host', { host: server.host_tag })
  } catch (error) {
    showMessage('连接失败', `无法连接到 ${server.host_tag}: ${error}`)
  }
}

// 编辑服务器状态
const showEditServerForm = ref(false)
const editingServer = ref({
  index: -1,
  data: {
    host_tag: '',
    hostname: '',
    user: '',
    port: 22,
    group: '',
    tagsInput: '',
    localForward: {
      local_port: '',
      remote_host: '',
      remote_port: ''
    },
    forwardAgent: false,
    dynamicForward: '',
    proxyJump: ''
  }
})

// 编辑服务器
function edit(server, index) {
  if (isFiltered.value) {
    showMessage('操作禁止', '过滤状态下不能编辑服务器')
    return
  }
  console.log('编辑服务器:', server)
  editingServer.value = {
    index,
    data: {
      ...server,
      tagsInput: server.tags ? server.tags.join(', ') : '',
      forwardAgent: server.forward_agent || false,
      dynamicForward: server.dynamic_forward || '',
      localForward: server.local_forward || {
        local_port: '',
        remote_host: '',
        remote_port: ''
      },
      proxyJump: server.proxy_jump || ''
    }
  }
  showEditServerForm.value = true
}

// 保存编辑
async function saveEditedServer() {
  console.log('保存:', editingServer.value)
  // 转换标签输入为数组
  const tags = editingServer.value.data.tagsInput
    ? editingServer.value.data.tagsInput.split(',').map(tag => tag.trim())
    : []
  
  let localForward = null
  let dynamicForward = null
  let proxyJump = null

  if (editingServer.value.data.forwardAgent) {
    dynamicForward = editingServer.value.data.dynamicForward
    console.log('server local forward:', editingServer.value.data.localForward)
    if (editingServer.value.data.localForward === '' || editingServer.value.data.localForward === null) {
      localForward = null
    } else {
      if (
      (editingServer.value.data.localForward.local_port !== '' || editingServer.value.data.localForward.local_port === null) && 
      (editingServer.value.data.localForward.remote_host !== '' || editingServer.value.data.localForward.remote_host === null) && 
      (editingServer.value.data.localForward.remote_port !== '' || editingServer.value.data.localForward.remote_port === null)) {
      localForward = {
        local_port: parseInt(editingServer.value.data.localForward.local_port, 10),
        remote_host: editingServer.value.data.localForward.remote_host,
        remote_port: parseInt(editingServer.value.data.localForward.remote_port, 10)
      }
    }
    }
    
    console.log('本地转发配置:', localForward)
    if (localForward === null && (dynamicForward === null || dynamicForward === '')) {
      showMessage('错误', '转发配置不完整，动态转发或者本地转发二选一必须填写一个')
      return
    }
  }
  
  // 更新服务器数据
  configData.value.servers[editingServer.value.index] = {
  host_tag: editingServer.value.data.host_tag,
  user: editingServer.value.data.user,
  hostname: editingServer.value.data.hostname,
  port: editingServer.value.data.port || 22,
  group: editingServer.value.data.group,
  tags: tags,
  forward_agent: editingServer.value.data.forwardAgent,
  dynamic_forward: dynamicForward,
  local_forward: localForward,
  proxy_jump: proxyJump
  }
  
  // 重置表单并关闭
  editingServer.value = {
    index: -1,
    data: {
      host_tag: '',
      hostname: '',
      user: '',
      port: 22,
      group: '',
      tagsInput: '',
      forwardAgent: false,
      dynamicForward: '',
      localForward: {
        local_port: '',
        remote_host: '',
        remote_port: ''
      },
      proxyJump: ''
    }
  }
  showEditServerForm.value = false
  console.log('保存编辑后的服务器:', configData.value.servers[editingServer.value.index])
  // 保存配置
  await saveConfig()
}

// 添加新主机
async function addNewServer() {
  // 转换标签输入为数组
  const tags = newServer.value.tagsInput
    ? newServer.value.tagsInput.split(',').map(tag => tag.trim())
    : []
  if (newServer.value.localForward.local_port != '') {
    // change string to int
    newServer.value.localForward.local_port = parseInt(newServer.value.localForward.local_port, 10)
  }
  let localForward = null
  let dynamicForward = null
  let proxyJump = null
  if (newServer.value.forwardAgent) {
    dynamicForward = newServer.value.dynamicForward
    console.log('server local forward:', newServer.value.localForward)
    if (newServer.value.localForward.local_port !== '' && newServer.value.localForward.remote_host !== '' && newServer.value.localForward.remote_port !== '') {
      localForward = {
        local_port: parseInt(newServer.value.localForward.local_port,10),
        remote_host: newServer.value.localForward.remote_host,
        remote_port: parseInt(newServer.value.localForward.remote_port,10)
      }
    }
    console.log('本地转发配置:', localForward)
    if (localForward === null && (dynamicForward === null || dynamicForward === '')) {
      showMessage('错误', '转发配置不完整，动态转发或者本地转发二选一必须填写一个')
      return
    }
  }
  // 添加新服务器
  configData.value.servers.push({
    host_tag: newServer.value.host_tag,
    user: newServer.value.user,
    hostname: newServer.value.hostname,
    port: newServer.value.port || 22,
    group: newServer.value.group,
    tags: tags,
    forward_agent: newServer.value.forwardAgent,
    dynamic_forward: dynamicForward,
    local_forward: localForward,
    proxy_jump: proxyJump
  })
  
  // 重置表单并关闭
  resetNewServerForm()
  showAddServerForm.value = false
  
  // 保存配置
  await saveConfig()
}

function resetNewServerForm() {
  newServer.value = {
    host_tag: '',
    hostname: '',
    user: '',
    port: 22,
    group: '',
    tagsInput: '',
    forwardAgent: false,
    dynamicForward: '',
    localForward: {
      local_port: '',
      remote_host: '',
      remote_port: ''
    },
    proxyJump: ''
    }
}

// 组件挂载时加载配置
onMounted(() => {
  loadConfig()
})
</script>

<style scoped>
.server-management {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.global-config-section, .server-list-section {
  background-color: #fff;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.config-card {
  background-color: #f8f9fa;
  border-radius: 6px;
  padding: 15px;
  margin-bottom: 15px;
}

.config-item {
  padding: 8px 0;
  border-bottom: 1px solid #eee;
}

.config-item:last-child {
  border-bottom: none;
}

/* 桌面视图 */
.desktop-view {
  display: block;
}

/* 移动视图 - 默认隐藏 */
.mobile-view {
  display: none;
}

.server-table {
  width: 100%;
  border-collapse: collapse;
}

.server-table th, .server-table td {
  border: 1px solid #ddd;
  padding: 12px;
  text-align: left;
}

.server-table th {
  background-color: #f2f2f2;
}

/* 服务器卡片样式 */
.server-card {
  background-color: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  margin-bottom: 15px;
  overflow: hidden;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 15px;
  background-color: #f8f9fa;
  border-bottom: 1px solid #eee;
}

.server-id {
  font-size: 0.9em;
  color: #666;
}

.server-tag {
  font-weight: bold;
}

.card-body {
  padding: 15px;
}

.info-row {
  display: flex;
  margin-bottom: 10px;
}

.info-row:last-child {
  margin-bottom: 0;
}

.label {
  font-weight: bold;
  min-width: 80px;
  margin-right: 10px;
  color: #333;
}

.card-actions {
  display: flex;
  padding: 10px 15px;
  background-color: #f8f9fa;
  border-top: 1px solid #eee;
}

.card-actions button {
  margin-right: 10px;
  padding: 6px 12px;
  font-size: 0.9em;
}

.add-button {
  background-color: #4CAF50;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  margin-right: 10px;
}

.add-button:hover {
  background-color: #45a049;
}

.save-button {
  background-color: #2196F3;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.save-button:hover {
  background-color: #0b7dda;
}

.delete-button {
  background-color: #f44336;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  margin-left: 5px;
}

.delete-button:hover {
  background-color: #d32f2f;
}

/* 模态框样式 */
.modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0,0,0,0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background-color: #fff;
  padding: 25px;
  border-radius: 8px;
  width: 500px;
  max-width: 90%;
  position: relative;
}

.close {
  position: absolute;
  top: 15px;
  right: 15px;
  font-size: 24px;
  cursor: pointer;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
}

.form-group input {
  width: 100%;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

button[type="submit"] {
  background-color: #2196F3;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
}

.tag {
  display: inline-block;
  background-color: #e0e0e0;
  border-radius: 4px;
  padding: 4px 10px;
  margin-right: 5px;
  font-size: 0.9em;
}

/* 消息模态框样式 */
.message-modal {
  max-width: 400px;
  text-align: center;
}

.message-header {
  padding: 15px 20px 10px;
  border-bottom: 1px solid #eee;
}

.message-header h4 {
  margin: 0;
  color: #333;
  font-size: 18px;
}

.message-body {
  padding: 20px;
}

.message-body p {
  margin: 0;
  color: #666;
  line-height: 1.5;
}

.message-footer {
  padding: 10px 20px 20px;
}

.confirm-button {
  background-color: #2196F3;
  color: white;
  border: none;
  padding: 10px 30px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  min-width: 80px;
}

.confirm-button:hover {
  background-color: #0b7dda;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .desktop-view {
    display: none;
  }
  
  .mobile-view {
    display: block;
  }
  
  .server-management {
    gap: 15px;
  }
  
  .global-config-section, .server-list-section {
    padding: 15px;
  }
}

/* 过滤区域样式 */
.filter-section {
  padding: 15px;
  background: #f5f5f5;
  border-radius: 8px;
  margin-bottom: 20px;
  display: flex;
  flex-wrap: wrap;
  gap: 15px;
  align-items: center;
}

.filter-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-control label {
  font-weight: bold;
  white-space: nowrap;
}

.filter-control select {
  padding: 6px 12px;
  border-radius: 4px;
  border: 1px solid #ddd;
  min-width: 120px;
}

.reset-button {
  padding: 6px 12px;
  background: #e0e0e0;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

.reset-button:hover {
  background: #d0d0d0;
}

@media (max-width: 480px) {
  .server-card {
    margin-bottom: 10px;
  }
  
  .card-header, .card-body, .card-actions {
    padding: 10px;
  }
  
  .info-row {
    flex-direction: column;
    margin-bottom: 8px;
  }
  
  .label {
    margin-bottom: 3px;
    min-width: auto;
  }
  
  .section-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  
  .section-header > div {
    display: flex;
    gap: 10px;
  }
  
  .add-button, .save-button {
    padding: 6px 12px;
    font-size: 13px;
  }
}
</style>