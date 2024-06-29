<script lang="ts">
export default {
  components: {
  },
}

</script>

<template>
  <div class="wrapper">
    <form action="/login" method="POST">
      <input type="text" name="username">
      <input type="password" name="password">
      <input type="submit">
    </form>
  </div>
</template>

<style scoped>
.right-panel {
  width: 300px;
}

.wrapper {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  width: 100vw;
}
</style>
