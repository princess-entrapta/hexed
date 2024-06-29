<script lang="ts">
import { useGridStore } from '@/stores/grid';
import Tile from './Tile.vue';

export default {
    emits: ["targetSelect"],
    computed: {
        grid() {
            return useGridStore().flat_grid
        },
        isPlaying() {
            return useGridStore().isPlaying
        },
        creatures() {
            return useGridStore().entities
        }
    },
    methods: {
    },
    components: {
        Tile,
    },
}
</script>

<template>
    <div class="container">
        <Tile v-for="[x, y, tileDef] in   grid  " :x="x" :y="y" :tileType="tileDef.type" :shadowed="!tileDef.visible"
            :entity="((creatures[x] || {})[y] || null)">
        </Tile>
    </div>
    <div v-if="!isPlaying">
        Waiting
    </div>
</template>

<style scoped>
.row {
    position: relative;
    display: inline-block;
    top: 0;
    left: 0;
}

.container {
    max-width: calc(100vw - 600px);
    height: calc(90vh - 74px);
    overflow: auto;
    position: relative;
}
</style>
