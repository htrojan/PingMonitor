<script setup>
import {onMounted, ref} from "vue";
import Plotly from "plotly.js-dist"
import {invoke} from '@tauri-apps/api/tauri'
import {emit, listen} from '@tauri-apps/api/event'

const graphic = ref('graphic')

onMounted(async () => {
  let data = {
    x: [],
    y: []
  };
  let layout = {
    margin: {t: 0}
  };

  Plotly.newPlot(graphic.value, [data], layout);

  const unlisten = await listen('ping', event => {
    // event.event is the event name (useful if you want to use a single callback fn for multiple event types)
    // event.payload is the payload object
    let date = new Date(event.payload["time"])
    let ping = parseInt(event.payload["ping"])
    console.log(date, ping)
    console.log("Ping!", event.payload);
    data.x.push(date)
    data.y.push(ping)
    if (data.x.length > 60) {
      data.x.shift()
      data.y.shift()
    }
    Plotly.update(graphic.value, [data], layout)
  })

})
</script>

<template>
  <h1>Ping</h1>
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
