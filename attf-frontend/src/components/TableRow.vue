<script lang="ts">
import { useGridStore } from '@/stores/grid'

export default {
    props: ['highlit', 'creatureData', 'selected', 'isActive'],
    data() {
        return { option: 'health', amount: 0 }
    },
    methods: {
        modify() {
            this.creatureData[this.option] += this.amount
            if (this.creatureData.health > this.creatureData.maxHealth) this.creatureData.health = this.creatureData.maxHealth
            if (this.creatureData.energy > this.creatureData.maxEnergy) this.creatureData.energy = this.creatureData.maxEnergy
            var grid = useGridStore()
            grid.updateView(this.creatureData)
        },
        endOfTurn() {
            var grid = useGridStore()
            grid.onTrait += 1
            if (grid.onTrait >= grid.creaturesByInitiative.length) {
                grid.onTrait = 0
            }
            this.creatureData.energy += this.creatureData.energyRegen
            if (this.creatureData.energy > this.creatureData.maxEnergy) this.creatureData.energy = this.creatureData.maxEnergy
            for (var statusIdx = 0; statusIdx < this.creatureData.statusData.length; statusIdx++) {
                this.creatureData.statusData[statusIdx].duration -= 1
                if (this.creatureData.statusData[statusIdx].duration <= 0) {
                    this.creatureData.statusData.splice(statusIdx, 1)
                }
            }
        }
    }
}
</script>

<template>
    <div :class="'table-row' + (highlit ? ' highlit' : '')">
        <div class="avatar-row">
            <img :src="creatureData.avatar" class="avatar">
            <div>
                <div>
                    <div class="bar">
                        <span>{{ creatureData.health + '/' + creatureData.maxHealth }}</span>
                        <div class="health" :style="'width:' + (creatureData.health / creatureData.maxHealth * 100) + '%'">
                        </div>
                    </div>
                    <div class="bar">
                        <span>{{ creatureData.energy + '/' + creatureData.maxEnergy }}</span>
                        <div class="energy" :style="'width:' + (creatureData.energy / creatureData.maxEnergy * 100) + '%'">
                        </div>
                    </div>
                </div>
                <div v-if="isActive" class="turn-row">
                    <img src="../assets/action-bar.png">
                    <button :onclick="endOfTurn">End of turn</button>
                </div>
            </div>
        </div>

        <div v-if="selected" class="action-row">
            <select v-model="option">
                <option>health</option>
                <option>maxHealth</option>
                <option>maxEnergy</option>
                <option>energyRegen</option>
                <option>energy</option>
                <option>initiative</option>
                <option>perception</option>
            </select>
            <input v-model="amount" type="number" class="command-line">
            <button @click="modify()">OK</button>
        </div>
    </div>
</template>

<style scoped>
select {
    background-color: #444;
    color: white
}

.turn-row {
    display: flex;
    justify-content: space-around;
    margin-top: 2px;
    height: 30px;
}

button {
    background-color: #444;
    color: #aaa;
    cursor: pointer;
}

.action-row {
    margin-top: 10px;
    margin-bottom: 10px;
    display: flex;
    justify-content: space-between;
}

.table-row {
    display: flex;
    flex-direction: column;
    background-color: #222;
    color: white;
    margin-bottom: 10px;
}

.avatar-row {
    display: flex;
}

.avatar-row>div {
    display: flex;
    flex-direction: column;
    width: 100%;
}

.avatar-row div div {
    display: flex;
}

.command-line {
    background-color: #444;
    color: white;
    width: 70px;
}

.highlit {
    background-color: #666;
}

.avatar {
    width: 64px;
    height: 64px;
    display: inline-block;
}

.bar {
    border: 1px solid white;
    height: 28px;
    background-color: black;
    flex: 1;
    display: flex;
    position: relative;
    text-align: center;
    color: white;
    line-height: 28px;
    z-index: 1;
}

.bar span {
    width: 100%;
    flex: 1;
}

.health {
    background-color: #a22;
    height: 100%;
    position: absolute;
    z-index: -1;
    top: 0
}

.energy {
    background-color: rgb(34, 126, 151);
    height: 100%;
    position: absolute;
    z-index: -1;
    top: 0
}</style>