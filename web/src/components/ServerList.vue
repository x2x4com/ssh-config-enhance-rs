<template>
  <div class="server-management">
    <!-- 全局配置区域 -->
    <div class="global-config-section">
      <h3>全局配置</h3>
      <div class="config-card">
        <div v-for="(value, key) in globalConfig" :key="key" class="config-item">
          <strong>{{ key }}:</strong> {{ value }}
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
        <button @click="showAddServerForm = true" class="add-button">
          添加新主机
        </button>
      </div>
      
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
          <tr v-for="(server, index) in servers" :key="index">
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
              <button @click="connect(server)">连接</button>
              <button @click="edit(server)">编辑</button>
            </td>
          </tr>
        </tbody>
      </table>
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
          <button type="submit">添加</button>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'

// 临时静态数据，后续替换为API调用
const servers = ref([
  {
    host_tag: 'example1',
    user: 'user1',
    hostname: 'host1.example.com',
    port: 22,
    group: '开发服务器',
    tags: ['dev', 'test']
  },
  {
    host_tag: 'example2',
    user: 'user2',
    hostname: 'host2.example.com',
    port: 22,
    group: '生产服务器',
    tags: ['prod']
  }
])

// 全局配置数据
const globalConfig = ref({
  'ServerAliveInterval': '30'
})

// 新服务器表单状态
const showAddServerForm = ref(false)
const newServer = ref({
  host_tag: '',
  hostname: '',
  user: '',
  port: 22,
  group: '',
  tagsInput: ''
})

// 新全局配置表单状态
const showAddGlobalConfig = ref(false)
const newConfig = ref({
  key: '',
  value: ''
})

function connect(server) {
  console.log('连接到服务器:', server.host_tag)
  // TODO: 调用Tauri命令连接
}

function edit(server) {
  console.log('编辑服务器:', server.host_tag)
  // TODO: 实现编辑功能
}

function addNewServer() {
  // 转换标签输入为数组
  const tags = newServer.value.tagsInput
    ? newServer.value.tagsInput.split(',').map(tag => tag.trim())
    : []
  
  // 添加新服务器
  servers.value.push({
    host_tag: newServer.value.host_tag,
    user: newServer.value.user,
    hostname: newServer.value.hostname,
    port: newServer.value.port || 22,
    group: newServer.value.group,
    tags: tags
  })
  
  // 重置表单并关闭
  resetNewServerForm()
  showAddServerForm.value = false
  
  // TODO: 调用后端API保存配置
}

function addGlobalConfig() {
  globalConfig.value[newConfig.value.key] = newConfig.value.value
  newConfig.value = { key: '', value: '' }
  showAddGlobalConfig.value = false
  
  // TODO: 调用后端API保存全局配置
}

function resetNewServerForm() {
  newServer.value = {
    host_tag: '',
    hostname: '',
    user: '',
    port: 22,
    group: '',
    tagsInput: ''
  }
}
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

.add-button {
  background-color: #4CAF50;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.add-button:hover {
  background-color: #45a049;
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
</style>