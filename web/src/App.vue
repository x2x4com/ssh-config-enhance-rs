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
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}
</style>