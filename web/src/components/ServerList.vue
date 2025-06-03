<template>
  <div class="server-list">
    <table>
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
</template>

<script setup>
import { ref } from 'vue'

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

function connect(server) {
  console.log('连接到服务器:', server.host_tag)
  // TODO: 调用Tauri命令连接
}

function edit(server) {
  console.log('编辑服务器:', server.host_tag)
  // TODO: 实现编辑功能
}
</script>

<style scoped>
.server-list {
  margin-top: 20px;
}
table {
  width: 100%;
  border-collapse: collapse;
}
th, td {
  border: 1px solid #ddd;
  padding: 8px;
  text-align: left;
}
th {
  background-color: #f2f2f2;
}
button {
  margin-right: 5px;
}
.tag {
  display: inline-block;
  background-color: #e0e0e0;
  border-radius: 4px;
  padding: 2px 8px;
  margin-right: 5px;
  font-size: 0.9em;
}
</style>