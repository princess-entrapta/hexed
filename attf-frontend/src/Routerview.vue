<script lang="ts">
import { useGridStore, useMouseInfoStore } from './stores/grid';
import RouterView from 'vue-router'
export default {
  data() {
    return { mouseStore: useMouseInfoStore(), grid: useGridStore() }
  },
  components: {

  },
}

</script>

<template>
  <RouterView></RouterView>
</template>

<style scoped></style>
