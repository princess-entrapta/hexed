<script lang="ts">
import { useGridStore, useMouseInfoStore } from '@/stores/grid';

export default {
    data() {
        let grid = useGridStore()
        let mouse = useMouseInfoStore()
        return { grid, mouse }
    },
}

</script>



<template>
    <div class="inventory">
        <div class="slots">
            <div :class="'slot ' + (ab == grid.ability ? 'selected ' : '') + (!grid.ability && mouse.tileSelected && grid.proposedAbilities.indexOf(idx) != -1 ? 'proposed ' : '')"
                v-for="(ab, idx) in  grid.abilities "
                @click.stop="(ev) => { grid.ability = ab; if (mouse.tileSelected && grid.proposedAbilities.indexOf(idx) != -1) grid.useAbility(mouse.tileSelected.x, mouse.tileSelected.y) }">
                {{ ab.name }}
                <!-- <img v-if="selectedCreature && selectedCreature.actionData.length > i"
                    :src="selectedCreature.actionData[i].imgUrl"> 
                -->
            </div>
        </div>
        <div class="tooltip">
        </div>
    </div>
</template>



<style scoped>
.inventory {
    width: calc(100vw - 600px);
    margin-top: 20px;
    position: relative;
    overflow: visible;
    display: flex;
}

.inventory .slot.selected {
    background-color: #888;
}

.inventory .slot.proposed {
    background-color: green;
}

.img {
    width: 64px;
    height: 64px;
}

.tooltip {
    position: absolute;
    bottom: 80px;
    border-radius: 8px;
    border: 1px solid #666;
    background-color: #222;
    z-index: 5;
    color: white;
    padding: 5px 10px;
}

.inventory .slot {
    width: 64px;
    height: 64px;
    flex-shrink: 0;
    border: 2px solid #666;
    background-color: #222;
    position: relative;
}


.slot span {
    position: absolute;
    z-index: 1;
    bottom: 0;
    right: 0;
    color: white;
    line-height: 18px;
}

.slots {
    width: calc(100vw - 600px);
    height: 80px;
    display: flex;
    overflow: auto;

}
</style>