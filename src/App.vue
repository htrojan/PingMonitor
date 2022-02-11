<script setup>
import {onMounted, ref} from "vue";
import Plotly from "plotly.js-dist"
import {invoke} from '@tauri-apps/api/tauri'

  const graphic = ref('graphic')

  onMounted(() => {
    Plotly.newPlot( graphic.value, [{
      x: [1, 2, 3, 4, 5],
      y: [1, 2, 4, 8, 16] }], {
      margin: { t: 0 } } );
  })
</script>

<template>
  <div ref="graphic"></div>
</template>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
