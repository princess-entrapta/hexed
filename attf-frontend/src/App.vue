<script lang="ts">
import Grid from './components/Grid.vue';
import { useGridStore, useMouseInfoStore } from './stores/grid';
import LeftPanel from './components/LeftPanel.vue';
import TableRow from './components/TableRow.vue';
import Inventory from './components/Inventory.vue'
import { reactive } from 'vue';
export default {
  props: ["gameId"],
  data() {
    let grid = useGridStore()
    grid.connect(this.gameId)
    let abilities = reactive([])
    return { mouseStore: useMouseInfoStore(), grid, abilities }
  },
  methods: {

  },
  components: {
    Grid,
    LeftPanel,
    TableRow,
    Inventory,
  },
}

</script>

<template>
  <RouterView></RouterView>
  <div class="wrapper" @click="mouseStore.clear()">
    <LeftPanel></LeftPanel>
    <div>
      <Grid />
      <Inventory :abilities="abilities"></Inventory>
    </div>
  </div>
</template>

<style scoped>
.right-panel {
  width: 300px;
}

.wrapper {
  display: flex;
  flex-direction: row;
  width: 100vw;
}
</style>
