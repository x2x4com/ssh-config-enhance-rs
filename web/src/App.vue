<template>
  <div class="app">
    <h1>SSH配置管理器</h1>
    <ServerList />
  </div>
</template>

<script setup>
import ServerList from './components/ServerList.vue'
</script>

<style>
.app {
  font-family: Arial, sans-serif;
  width: 100%;
  margin: 0 auto;
  padding: 20px;
}

@media (max-width: 768px) {
  .app {
    padding: 15px;
  }
}

@media (max-width: 480px) {
  .app {
    padding: 10px;
  }
}
</style>