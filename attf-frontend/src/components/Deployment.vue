<script setup lang="ts">
import { defineProps } from 'vue'
import { useDeployStore } from '@/stores/deploy.ts'
import { coordsFromString } from '@/stores/grid'
import Tile from './Tile.vue'
import Creature from './Creature.vue'

const props = defineProps(["tiles", "entities", "deployedEntities",])
const components = {
    Tile,
    Creature
}
const store = useDeployStore()
</script>

<template>
    <div class="container">
        <div class="left-panel">
            <div class="no-entity" @click="store.selectedCreature = null">none</div>
            <Creature v-for="entity in entities" :game_class="entity.game_class" @click="store.selectedCreature = entity">
            </Creature>
        </div>
        <div class="grid-panel">
            <Tile v-for="tile in tiles" :x="coordsFromString(tile).x" :y="coordsFromString(tile).y" tileType="Floor"
                :entity="deployedEntities[coordsFromString(tile).x][coordsFromString(tile).y]"
                @click="deployedEntities[coordsFromString(tile).x][coordsFromString(tile).y] = store.selectedCreature">
            </Tile>
        </div>
        <button @click="store.deploy">Deploy and start game</button>
    </div>
</template>

<style scoped>
.left-panel {
    width: 80px;
    border-right: 1px solid rgba(255, 255, 255, 0.3);
    display: flex;
    flex-direction: column;
}

.no-entity {
    border: 1px solid #888;
    line-height: 60px;
    text-align: center;
}

.left-panel * {
    width: 60px;
    height: 60px;
}

.grid-panel {
    width: 70vw;
    height: 80vh;
}

.container {
    display: flex;
    margin-top: 20px;
}
</style>