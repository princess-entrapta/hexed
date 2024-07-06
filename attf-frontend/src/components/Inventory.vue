<script lang="ts">
import { useGridStore, useMouseInfoStore, isAbilityReady } from '@/stores/grid';

export default {
    data() {
        let grid = useGridStore()
        let mouse = useMouseInfoStore()
        return { grid, mouse }
    },
    methods: {
        ready(ab) {
            return isAbilityReady(ab)
        }
    }
}

</script>



<template>
    <div class="inventory">
        <div class="slots">
            <div :class="'slot ' + (ab == grid.ability ? 'selected ' : '') + (!grid.ability && mouse.tileSelected && grid.proposedAbilities.indexOf(idx) != -1 ? 'proposed ' : '') + (ready(ab) ? '' : 'disabled ')"
                v-for="(ab, idx) in  grid.abilities "
                @click.stop="(ev) => { grid.ability = ab; if (grid.ability.targets.length == 1) {mouse.tileSelected = grid.ability.targets[0]} if (mouse.tileSelected && grid.proposedAbilities.indexOf(idx) != -1) grid.useAbility(mouse.tileSelected.x, mouse.tileSelected.y) }">
                {{ ab.name }} {{ }}
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
    background-color: #2a8;
    color: white;
}

.inventory .slot.proposed {
    background-color: green;
    color: white
}

.inventory .slot.disabled {
    background-color: #666;
    color: #999;
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